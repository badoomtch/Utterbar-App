/**
 * Mode icon renderer.
 *
 * The Rust side persists each mode's icon as a string (`mic`,
 * `message-square`, etc — the kebab-case identifiers lucide-react
 * documentation uses). The frontend used to render those as emoji
 * fallbacks via a hardcoded lookup, which silently broke for any icon
 * not in the lookup map. This component renders them properly via
 * lucide-react.
 *
 * Two surfaces consume this:
 *
 *   - <ModeIcon name={mode.icon} /> — list item / dropdown / floating-bar
 *     mode-name chip. Just renders the icon.
 *   - <ModeIconPicker value, onChange /> — editor grid picker. Renders
 *     a 6-wide grid of curated icons; clicking one calls onChange with
 *     the canonical kebab-case name.
 *
 * Icon registry below is curated, not exhaustive. Lucide ships ~600
 * icons; we expose ~24 mapped to plausible mode roles. The registry is
 * the source of truth: anything not in it falls back to Sparkles in
 * the renderer (existing custom modes with stale icon names render as
 * Sparkles rather than crashing).
 */
import {
  // Communication
  Mic,
  MessageSquare,
  MessagesSquare,
  Mail,
  Phone,
  Send,
  // Writing
  PenLine,
  FileText,
  BookOpen,
  Newspaper,
  StickyNote,
  Quote,
  // Work contexts
  Briefcase,
  Code2,
  Terminal,
  Calendar,
  Users,
  Presentation,
  // General / abstract
  Sparkles,
  Star,
  Zap,
  Hash,
  Bookmark,
  CheckSquare,
  type LucideIcon,
} from 'lucide-react';

import { cn } from '@/lib/cn';

/**
 * Curated icon registry. Keyed by the kebab-case identifier that the
 * Rust side persists (matches the lucide-react documentation). Order
 * here is the order they appear in the picker grid, grouped roughly
 * by use-case so the user can scan quickly.
 *
 * To add an icon:
 *   1. Import the component at the top of this file.
 *   2. Add a `[kebab-name]: Component` entry below.
 *   3. The picker grid auto-renders it.
 */
const ICON_REGISTRY: Record<string, LucideIcon> = {
  // Communication — conversational, message-flavoured
  mic: Mic,
  'message-square': MessageSquare,
  'messages-square': MessagesSquare,
  mail: Mail,
  phone: Phone,
  send: Send,
  // Writing — text production
  'pen-line': PenLine,
  'file-text': FileText,
  'book-open': BookOpen,
  newspaper: Newspaper,
  'sticky-note': StickyNote,
  quote: Quote,
  // Work contexts — domain modes
  briefcase: Briefcase,
  'code-2': Code2,
  terminal: Terminal,
  calendar: Calendar,
  users: Users,
  presentation: Presentation,
  // General / abstract — fallback / utility
  sparkles: Sparkles,
  star: Star,
  zap: Zap,
  hash: Hash,
  bookmark: Bookmark,
  'check-square': CheckSquare,
};

/** All icon keys, in registry/grid order. Used by the picker. */
export const MODE_ICON_KEYS = Object.keys(ICON_REGISTRY);

/** Default icon when a mode persists an unknown name. */
const DEFAULT_ICON: LucideIcon = Sparkles;

/**
 * Render a mode's icon. `name` is the kebab-case lucide identifier
 * (e.g. `mic`, `message-square`). Unknown names fall back to Sparkles
 * — a deliberately neutral marker so the row still renders.
 */
export function ModeIcon({
  name,
  className,
  size = 16,
}: {
  name: string;
  className?: string;
  size?: number;
}) {
  const Icon = ICON_REGISTRY[name] ?? DEFAULT_ICON;
  return <Icon className={className} aria-hidden size={size} strokeWidth={1.75} />;
}

/**
 * Grid picker. Renders all curated icons; clicking one calls onChange
 * with the canonical kebab-case key. The currently-selected icon gets
 * a cyan ring matching the design system's active-state pattern.
 *
 * Callers wrap this in a Field; this component is just the grid.
 */
export function ModeIconPicker({
  value,
  onChange,
}: {
  value: string;
  onChange: (next: string) => void;
}) {
  return (
    <div className="grid grid-cols-6 gap-2">
      {MODE_ICON_KEYS.map((key) => {
        const active = key === value;
        const Icon = ICON_REGISTRY[key]!;
        return (
          <button
            key={key}
            type="button"
            onClick={() => onChange(key)}
            aria-label={key}
            aria-pressed={active}
            title={key}
            className={cn(
              'flex h-10 w-10 items-center justify-center rounded-input transition-colors',
              active
                ? 'bg-bg-0 ring-2 ring-accent text-fg-hi'
                : 'bg-bg-0 text-muted hover:bg-bg-1 hover:text-fg-hi',
            )}
          >
            <Icon size={18} strokeWidth={1.75} aria-hidden />
          </button>
        );
      })}
    </div>
  );
}
