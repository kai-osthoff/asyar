<script lang="ts">
  import { onMount } from 'svelte';
  import SettingsCard from './SettingsCard.svelte';
  import SettingsRow from './SettingsRow.svelte';
  import StatusDot from '../base/StatusDot.svelte';
  import Button from '../base/Button.svelte';
  import { accessibilityStatusService as svc } from '../../services/permissions/accessibilityStatus.svelte';

  const RESET_COMMAND = 'sudo tccutil reset Accessibility org.asyar.app';

  let repairMessage = $state<string | null>(null);
  let copied = $state(false);

  let dot = $derived(
    svc.status === 'granted'
      ? { color: 'success' as const, label: 'Granted' }
      : svc.status === 'stale_grant'
        ? { color: 'warning' as const, label: 'Not in effect' }
        : svc.status === 'unknown'
          ? { color: 'info' as const, label: 'Status unavailable' }
          : { color: 'warning' as const, label: 'Not granted' },
  );

  let description = $derived(
    svc.status === 'granted'
      ? 'Asyar can paste, expand snippets, and read your selection.'
      : svc.status === 'stale_grant'
        ? 'System Settings lists Asyar as enabled, but the entry was created for an older build of Asyar and no longer applies. macOS ties each entry to a code signature, so switching it off and on again will not help — the entry has to be removed and recreated.'
        : svc.status === 'unknown'
          ? 'Asyar could not read the permission state.'
          : 'Pasting from clipboard history and snippet expansion need this permission. Without it macOS silently discards the keystroke.',
  );

  async function copyCommand() {
    await navigator.clipboard.writeText(RESET_COMMAND);
    copied = true;
  }

  async function runRepair() {
    repairMessage = null;
    const result = await svc.repair();
    repairMessage = result.ok
      ? 'Entry removed. Now grant the permission again.'
      : (result.error ?? null);
  }

  onMount(() => {
    void svc.init();
    // The grant is made in another application, so the value can only have
    // changed while this window was in the background.
    const onFocus = () => void svc.refresh();
    window.addEventListener('focus', onFocus);
    return () => window.removeEventListener('focus', onFocus);
  });
</script>

<div class="section-header">Accessibility</div>
<SettingsCard>
  <SettingsRow label="macOS Accessibility" {description}>
    {#snippet children()}
      <div class="status-row">
        <StatusDot color={dot.color} />
        <span class="text-body">{dot.label}</span>
      </div>
    {/snippet}
  </SettingsRow>

  {#if svc.status !== 'granted'}
    <div class="actions">
      <Button onclick={() => svc.requestPermission()}>Grant permission</Button>
      <button class="link" onclick={() => svc.openPreferences()}>Open System Settings</button>
    </div>
  {/if}

  {#if svc.status === 'stale_grant'}
    <div class="repair">
      <ol class="steps">
        <li>
          In System Settings → Privacy &amp; Security → Accessibility, select Asyar and remove it
          with the “−” button.
        </li>
        <li>Quit and reopen Asyar.</li>
        <li>Grant the permission again with the button above.</li>
      </ol>
      <div class="command-row">
        <code>{RESET_COMMAND}</code>
        <button class="link" onclick={copyCommand}>{copied ? 'Copied' : 'Copy'}</button>
      </div>
      <div class="auto-repair">
        <Button onclick={runRepair}>
          Repair automatically (asks for your administrator password)
        </Button>
        {#if repairMessage}
          <p class="repair-message">{repairMessage}</p>
        {/if}
      </div>
    </div>
  {/if}
</SettingsCard>

<style>
  .status-row {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) 0 0;
  }
  .repair {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding-top: var(--space-4);
  }
  .steps {
    margin: 0;
    padding-left: var(--space-5);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
  }
  .command-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }
  .command-row code {
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    padding: var(--space-1) var(--space-2);
  }
  .auto-repair {
    border-top: 1px solid var(--separator);
    padding-top: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    align-items: flex-start;
  }
  .repair-message {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  .link {
    background: none;
    border: none;
    padding: 0;
    color: var(--asyar-brand);
    font-size: var(--font-size-sm);
    cursor: pointer;
  }
</style>
