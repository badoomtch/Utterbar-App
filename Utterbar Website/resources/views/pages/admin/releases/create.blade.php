<x-layouts.app title="Admin · New release" :no-index="true">
    <section class="pt-16 md:pt-20 pb-20">
        <div class="ub-container">
            <div class="max-w-5xl mx-auto">

                <x-admin-nav active="releases" />

                <div class="mb-8">
                    <h1 class="ub-fg-hi" style="font-size: clamp(2rem, 4vw, 2.75rem); letter-spacing: -0.035em; line-height: 1.05;">
                        New release.
                    </h1>
                    <p class="mt-3 text-sm" style="color: var(--muted);">
                        Upload the installer to R2 first, then create the release record here.
                    </p>
                </div>

                <form method="POST" action="{{ route('admin.releases.store') }}" class="ub-card space-y-5"
                      style="padding: 2rem;" x-data="{ version: '{{ old('version', '') }}' }">
                    @csrf

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label for="r-version" class="ub-eyebrow mb-2 block">Version</label>
                            <input id="r-version" name="version" type="text" required autofocus
                                   x-model="version"
                                   placeholder="0.5.0"
                                   value="{{ old('version') }}"
                                   class="w-full rounded-xl px-4 py-3 font-mono"
                                   style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">
                            @error('version')<p class="mt-1 text-xs" style="color: var(--danger);">{{ $message }}</p>@enderror
                        </div>
                        <div>
                            <label for="r-channel" class="ub-eyebrow mb-2 block">Channel</label>
                            <select id="r-channel" name="channel"
                                    class="w-full rounded-xl px-4 py-3"
                                    style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">
                                <option value="stable" {{ old('channel', 'stable') === 'stable' ? 'selected' : '' }}>stable</option>
                                <option value="beta"   {{ old('channel') === 'beta'   ? 'selected' : '' }}>beta</option>
                            </select>
                            @error('channel')<p class="mt-1 text-xs" style="color: var(--danger);">{{ $message }}</p>@enderror
                        </div>
                    </div>

                    <div>
                        <label for="r-key" class="ub-eyebrow mb-2 block">R2 object key</label>
                        <input id="r-key" name="r2_key" type="text" required
                               :placeholder="version ? 'releases/' + version + '/Utterbar-Setup.exe' : 'releases/0.5.0/Utterbar-Setup.exe'"
                               value="{{ old('r2_key') }}"
                               class="w-full rounded-xl px-4 py-3 font-mono"
                               style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 0.9rem; outline-color: var(--accent);">
                        <p class="mt-1 text-xs" style="color: var(--muted);">
                            Path inside the <span class="font-mono">utterbar</span> bucket. Must already exist.
                            Public URL: <span class="font-mono" style="color: var(--accent);" x-text="'https://dl.utterbar.com/' + ($el.value || 'path-you-type-above')"></span>
                        </p>
                        @error('r2_key')<p class="mt-1 text-xs" style="color: var(--danger);">{{ $message }}</p>@enderror
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label for="r-tag" class="ub-eyebrow mb-2 block">Tag <span style="color: var(--muted); text-transform: none; letter-spacing: 0; font-weight: 400;">(optional)</span></label>
                            <input id="r-tag" name="tag" type="text" maxlength="64"
                                   placeholder="latest preview"
                                   value="{{ old('tag') }}"
                                   class="w-full rounded-xl px-4 py-3"
                                   style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 1rem; outline-color: var(--accent);">
                            <p class="mt-1 text-xs" style="color: var(--muted);">Small cyan pill next to the version.</p>
                        </div>
                        <div>
                            <label for="r-sha" class="ub-eyebrow mb-2 block">SHA-256 <span style="color: var(--danger); text-transform: none; letter-spacing: 0; font-weight: 400;">(required)</span></label>
                            <input id="r-sha" name="sha256" type="text" required pattern="[A-Fa-f0-9]{64}"
                                   placeholder="64 hex chars"
                                   value="{{ old('sha256') }}"
                                   class="w-full rounded-xl px-4 py-3 font-mono"
                                   style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 0.75rem; outline-color: var(--accent);">
                            @error('sha256')<p class="mt-1 text-xs" style="color: var(--danger);">{{ $message }}</p>@enderror
                        </div>
                    </div>

                    <div>
                        <label for="r-notes" class="ub-eyebrow mb-2 block">Release notes (markdown)</label>
                        <textarea id="r-notes" name="notes" required rows="12"
                                  placeholder="- First note&#10;- Second note&#10;- Third note"
                                  class="w-full rounded-xl px-4 py-3 font-mono"
                                  style="background: var(--bg-0); border: 1px solid var(--border-strong); color: var(--fg); font-size: 0.9rem; line-height: 1.6; outline-color: var(--accent);">{{ old('notes') }}</textarea>
                        @error('notes')<p class="mt-1 text-xs" style="color: var(--danger);">{{ $message }}</p>@enderror
                    </div>

                    <div class="rounded-xl" style="background: var(--bg-0); border: 1px solid var(--border-strong); padding: 1rem 1.25rem;">
                        <label class="flex items-start gap-3 cursor-pointer">
                            <input type="checkbox" name="announce" value="1" {{ old('announce') ? 'checked' : '' }}
                                   class="mt-0.5 shrink-0 w-4 h-4 rounded cursor-pointer"
                                   style="accent-color: var(--accent);">
                            <span>
                                <span style="color: var(--fg-hi); font-weight: 500;">Email subscribers about this release</span>
                                <span class="block text-xs mt-0.5" style="color: var(--muted);">
                                    Sends the release notes to every confirmed release-update subscriber. Only works for stable channel.
                                </span>
                            </span>
                        </label>
                        <div class="mt-3">
                            <label for="r-subj" class="ub-eyebrow mb-2 block" style="font-size: 10.5px;">Announcement subject</label>
                            <input id="r-subj" name="announce_subject" type="text" maxlength="200"
                                   :placeholder="version ? 'Utterbar ' + version + ' is out' : 'Utterbar 0.5.0 is out'"
                                   value="{{ old('announce_subject') }}"
                                   class="w-full rounded-xl px-4 py-3"
                                   style="background: var(--bg-1); border: 1px solid var(--border-strong); color: var(--fg); font-size: 0.9rem; outline-color: var(--accent);">
                        </div>
                    </div>

                    <div class="flex items-center justify-end gap-3 pt-2">
                        <a href="{{ route('admin.releases.index') }}"
                           class="inline-flex items-center rounded-lg"
                           style="padding: 0.55rem 1rem; font-size: 0.875rem; font-weight: 500; background: transparent; border: 1px solid var(--border-strong); color: var(--fg);">
                            Cancel
                        </a>
                        <button type="submit" class="ub-btn ub-btn-cta">Publish release</button>
                    </div>
                </form>

            </div>
        </div>
    </section>
</x-layouts.app>
