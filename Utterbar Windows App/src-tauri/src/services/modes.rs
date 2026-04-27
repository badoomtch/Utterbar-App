//! Mode CRUD + built-in seed.

use std::sync::Arc;

use anyhow::Result;
use rusqlite::params;

use crate::db::Db;
use crate::domain::{
    AutoActivationRule, BulletStyle, Capitalisation, ContextRules, Formatting, Mode, PostProcessing,
};
use crate::util::time::now_ms;

pub struct ModeService {
    db: Arc<Db>,
}

impl ModeService {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub fn ensure_seed(&self) -> Result<()> {
        // 0.6 cleanup: the LLM built-ins (Email / Note / Super) shipped in
        // pre-release builds but their LLM provider was never wired, so they
        // silently fell back to deterministic output. We defer LLM
        // post-processing to 0.7 and drop the three modes from the built-in
        // catalogue — when LLM lands they return with real behaviour.
        //
        // Delete them unconditionally on every launch so dev-build upgraders
        // don't carry stale is_builtin=1 rows that the new catalogue no
        // longer seeds. Custom user modes are untouched (they'd have
        // is_builtin=0 and different IDs even if a user named one "Email").
        {
            let conn = self.db.conn()?;
            conn.execute(
                "DELETE FROM modes WHERE is_builtin=1 AND id IN ('email','note','super')",
                [],
            )?;
        }

        let conn = self.db.conn()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM modes", [], |r| r.get(0))?;
        if count > 0 {
            // 0.13 upgrade: existing DBs already have a built-in "meeting"
            // row but the V0003 ALTER set its is_meeting_mode to 0 (the
            // column default). Flip it on so users who upgrade without
            // wiping their DB get the new behaviour. Also clear any
            // post-processing template we previously seeded — meeting
            // mode skips post-processing at runtime, so a stale
            // deterministic template would just confuse anyone who opens
            // the editor expecting it to apply.
            let _ = conn.execute(
                "UPDATE modes
                    SET is_meeting_mode = 1,
                        post_processing = '{\"kind\":\"none\"}'
                  WHERE id = 'meeting' AND is_builtin = 1 AND is_meeting_mode = 0",
                [],
            );
            return Ok(());
        }
        drop(conn);
        for (i, seed) in builtin_modes().into_iter().enumerate() {
            self.upsert(seed, i as i64)?;
        }
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Mode>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare("SELECT id,name,icon,is_builtin,is_favourite,sort_order,asr_model_id,target_language,post_processing,context_rules,auto_activation,shortcut_override,formatting,english_variant,is_meeting_mode,created_at,updated_at FROM modes ORDER BY sort_order")?;
        let rows = stmt.query_map([], row_to_mode)?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get(&self, id: &str) -> Result<Option<Mode>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare("SELECT id,name,icon,is_builtin,is_favourite,sort_order,asr_model_id,target_language,post_processing,context_rules,auto_activation,shortcut_override,formatting,english_variant,is_meeting_mode,created_at,updated_at FROM modes WHERE id=?1")?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row_to_mode(row)?)),
            None => Ok(None),
        }
    }

    pub fn upsert(&self, mut mode: Mode, sort_order: i64) -> Result<Mode> {
        let now = now_ms();
        mode.updated_at = now;
        if mode.created_at == 0 {
            mode.created_at = now;
        }
        let conn = self.db.conn()?;
        // english_variant is serialised as plain snake_case ("auto" /
        // "british" / "american") so SQL queries can read it without
        // unwrapping JSON. serde_json wraps the enum in quotes by default,
        // so we trim them here.
        let variant = serde_json::to_string(&mode.english_variant)?
            .trim_matches('"')
            .to_string();
        conn.execute(
            "INSERT INTO modes(id,name,icon,is_builtin,is_favourite,sort_order,asr_model_id,target_language,post_processing,context_rules,auto_activation,shortcut_override,formatting,english_variant,is_meeting_mode,created_at,updated_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)
             ON CONFLICT(id) DO UPDATE SET
                name=excluded.name, icon=excluded.icon, is_favourite=excluded.is_favourite,
                sort_order=excluded.sort_order, asr_model_id=excluded.asr_model_id,
                target_language=excluded.target_language, post_processing=excluded.post_processing,
                context_rules=excluded.context_rules, auto_activation=excluded.auto_activation,
                shortcut_override=excluded.shortcut_override, formatting=excluded.formatting,
                english_variant=excluded.english_variant,
                is_meeting_mode=excluded.is_meeting_mode,
                updated_at=excluded.updated_at",
            params![
                mode.id, mode.name, mode.icon, mode.is_builtin as i64, mode.is_favourite as i64, sort_order,
                mode.asr_model_id, mode.target_language,
                serde_json::to_string(&mode.post_processing)?,
                serde_json::to_string(&mode.context_rules)?,
                serde_json::to_string(&mode.auto_activation)?,
                mode.shortcut_override.as_ref().map(|s| serde_json::to_string(s).unwrap()),
                serde_json::to_string(&mode.formatting)?,
                variant,
                mode.is_meeting_mode as i64,
                mode.created_at, mode.updated_at,
            ],
        )?;
        Ok(mode)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.db.conn()?;
        conn.execute(
            "DELETE FROM modes WHERE id=?1 AND is_builtin=0",
            params![id],
        )?;
        Ok(())
    }

    pub fn duplicate(&self, id: &str) -> Result<Mode> {
        let src = self
            .get(id)?
            .ok_or_else(|| anyhow::anyhow!("mode not found"))?;
        let mut copy = src.clone();
        copy.id = ulid::Ulid::new().to_string();
        copy.name = format!("{} Copy", src.name);
        copy.is_builtin = false;
        copy.created_at = 0;
        self.upsert(copy, i64::MAX / 2)
    }

    /// Walk every mode and return the first whose auto-activation rules match
    /// the given foreground-app signals.
    ///
    /// Precedence is intentionally simple and deterministic: rules are
    /// evaluated in `sort_order` (the order modes appear in the user's
    /// list — favourites bubble up via the same sort). The first mode
    /// whose first-matching rule fires is the winner. Within a single
    /// mode, rules are evaluated in array order; first hit wins. No
    /// scoring, no specificity, no priority field — if you want a mode
    /// to take precedence, drag it above the others in the editor.
    pub fn match_auto_activation(
        &self,
        exe: Option<&str>,
        title: Option<&str>,
        class: Option<&str>,
    ) -> Result<Option<Mode>> {
        for m in self.list()? {
            for rule in &m.auto_activation {
                let hit = match rule {
                    AutoActivationRule::ByExe { pattern } => {
                        exe.map(|v| glob(v, pattern)).unwrap_or(false)
                    }
                    AutoActivationRule::ByTitle { pattern } => {
                        title.map(|v| glob(v, pattern)).unwrap_or(false)
                    }
                    AutoActivationRule::ByClass { pattern } => {
                        class.map(|v| glob(v, pattern)).unwrap_or(false)
                    }
                };
                if hit {
                    return Ok(Some(m));
                }
            }
        }
        Ok(None)
    }
}

fn row_to_mode(row: &rusqlite::Row) -> rusqlite::Result<Mode> {
    let post: String = row.get(8)?;
    let ctx: String = row.get(9)?;
    let auto: String = row.get(10)?;
    let shortcut: Option<String> = row.get(11)?;
    let fmt: String = row.get(12)?;
    // `english_variant` is a short discriminant string ("auto" / "british" /
    // "american"). We serialise it via serde_json so the snake-case mapping
    // matches the rest of the codebase — just wrap it in quotes so the
    // deserialiser sees JSON.
    let variant_raw: String = row.get(13)?;
    let english_variant = serde_json::from_str(&format!("\"{}\"", variant_raw))
        .unwrap_or(crate::domain::EnglishVariant::Auto);
    Ok(Mode {
        id: row.get(0)?,
        name: row.get(1)?,
        icon: row.get(2)?,
        is_builtin: row.get::<_, i64>(3)? != 0,
        is_favourite: row.get::<_, i64>(4)? != 0,
        sort_order: row.get(5)?,
        asr_model_id: row.get(6)?,
        target_language: row.get(7)?,
        post_processing: serde_json::from_str(&post).unwrap_or(PostProcessing::None),
        context_rules: serde_json::from_str(&ctx).unwrap_or_default(),
        auto_activation: serde_json::from_str(&auto).unwrap_or_default(),
        shortcut_override: shortcut.and_then(|s| serde_json::from_str(&s).ok()),
        formatting: serde_json::from_str(&fmt).unwrap_or_default(),
        english_variant,
        is_meeting_mode: row.get::<_, i64>(14)? != 0,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

// Simple glob: supports `*` wildcards, case-insensitive.
fn glob(haystack: &str, pattern: &str) -> bool {
    let h = haystack.to_lowercase();
    let p = pattern.to_lowercase();
    let parts: Vec<&str> = p.split('*').collect();
    let mut idx = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 && !p.starts_with('*') {
            if !h[idx..].starts_with(part) {
                return false;
            }
            idx += part.len();
            continue;
        }
        match h[idx..].find(part) {
            Some(pos) => idx += pos + part.len(),
            None => return false,
        }
    }
    if !p.ends_with('*') {
        if let Some(last) = parts.last() {
            if !last.is_empty() && !h.ends_with(last) {
                return false;
            }
        }
    }
    true
}

fn builtin_modes() -> Vec<Mode> {
    let mk = |id: &str, name: &str, icon: &str, pp: PostProcessing, fmt: Formatting| Mode {
        id: id.into(),
        name: name.into(),
        icon: icon.into(),
        is_builtin: true,
        is_favourite: false,
        sort_order: 0,
        asr_model_id: None,
        target_language: None,
        post_processing: pp,
        context_rules: ContextRules::default(),
        auto_activation: Vec::new(),
        shortcut_override: None,
        formatting: fmt,
        // Ship seeded built-ins with Auto — the user's first interaction
        // with the feature is flipping a mode to British/American in the
        // mode editor, which then persists the override.
        english_variant: crate::domain::EnglishVariant::Auto,
        // Default off for built-ins. The seed below flips this on for the
        // built-in "meeting" row so it works out of the box.
        is_meeting_mode: false,
        created_at: 0,
        updated_at: 0,
    };

    let base_fmt = Formatting {
        capitalisation: Capitalisation::Sentence,
        strip_fillers: true,
        auto_punctuation: true,
        bullet_style: BulletStyle::None,
        preserve_linebreaks: false,
    };

    // 0.6 catalogue — three built-ins, all deterministic. Email / Note /
    // Super are parked until 0.7 when the llama.cpp provider actually works;
    // shipping them today would mean shipping `PostProcessing::Llm` modes
    // that silently fall back to deterministic at runtime. See
    // `ensure_seed` above for the one-shot cleanup that removes pre-release
    // copies from existing DBs.
    vec![
        mk(
            "voice-to-text",
            "Voice to Text",
            "mic",
            PostProcessing::None,
            base_fmt.clone(),
        ),
        mk(
            "message",
            "Message",
            "message-square",
            PostProcessing::Deterministic {
                template: "{text}".into(),
            },
            base_fmt.clone(),
        ),
        Mode {
            is_meeting_mode: true,
            ..mk(
                "meeting",
                "Meeting",
                "users",
                // Meeting mode skips post-processing at runtime
                // regardless of this field, but keeping it as None makes
                // the user's mental model match what actually happens —
                // they don't open the mode editor and see a deterministic
                // template that's silently ignored.
                PostProcessing::None,
                Formatting {
                    preserve_linebreaks: true,
                    ..base_fmt
                },
            )
        },
    ]
}
