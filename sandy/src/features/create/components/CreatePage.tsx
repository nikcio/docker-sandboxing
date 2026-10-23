import { useId } from "react";
import Icon from "../../../common/components/Icon";
import { ErrorBanner } from "../../../common/components/ErrorBanner";
import Providers from "../../settings/components/Providers";
import { useCreateFlow } from "../hooks/useCreateFlow";
import { DiscoveryStep } from "./DiscoveryStep";
import { AgentStep } from "./AgentStep";
import { MixinStep } from "./MixinStep";
import { ReviewStep } from "./ReviewStep";
import { FinishedPanel } from "./FinishedPanel";

export interface CreatePageProps {
  onClose: () => void;
  onTemplates?: () => void;
  onBusyChange?: (busy: boolean) => void;
}

const STEPS = ["Providers", "Agent or workload", "Mixins & inputs", "Project & review"];

export default function CreatePage({ onClose, onTemplates, onBusyChange }: CreatePageProps) {
  const id = useId();
  const flow = useCreateFlow({ onBusyChange });
  const {
    step,
    moveTo,
    busy,
    error,
    providers,
    hubQuery,
    setHubQuery,
    catalog,
    onProvidersChange,
    discover,
    agents,
    agent,
    agentSearch,
    setAgentSearch,
    selectAgent,
    compatible,
    matchingMixins,
    mixinGroups,
    mixins,
    mixinSearch,
    setMixinSearch,
    providerFilter,
    setProviderFilter,
    providerLabel,
    toggleMixin,
    removeMixin,
    changeArgument,
    inputValid,
    request,
    name,
    setName,
    workspace,
    setWorkspace,
    savePath,
    setSavePath,
    submit,
    browse,
    saved,
    launched,
    launchOutput,
    copyNotice,
    retryLaunch,
    copy,
  } = flow;

  return (
    <section className="standalone-page catalog-page" aria-labelledby={`${id}-title`} aria-busy={busy !== null}>
      <header className="page-head catalog-page-heading">
        <div>
          <span className="eyebrow">YOUR NEXT WORKSPACE</span>
          <h1 id={`${id}-title`}>
            {saved ? "Your template is ready" : "Create a sandbox"}
          </h1>
          <p className="page-sub">
            {saved
              ? "A reusable starting point, saved on your computer."
              : "Choose your tools. Bring your project. Make it yours."}
          </p>
        </div>
        <button type="button" className="ghost" disabled={busy !== null} onClick={onClose}>
          <Icon name="close" size={16} /> {saved ? "Close" : "Cancel"}
        </button>
      </header>
      {!saved && (
        <ol className="setup-steps catalog-steps" aria-label="Sandbox creation steps">
          {STEPS.map((label, index) => (
            <li key={label}>
              <button
                type="button"
                className={`setup-step ${step === index ? "current" : ""} ${step > index ? "complete" : ""}`}
                aria-current={step === index ? "step" : undefined}
                disabled={busy !== null || index > step}
                onClick={() => moveTo(index)}
              >
                <span className="step-number">
                  {step > index ? <Icon name="check" size={14} /> : index + 1}
                </span>
                {label}
              </button>
            </li>
          ))}
        </ol>
      )}
      {error && (
        <ErrorBanner
          title={saved ? "Launch needs attention." : "We couldn’t complete that step."}
          actionLabel={step === 0 && !saved ? "Retry discovery" : undefined}
          disabled={busy !== null || !providers}
          onAction={step === 0 && !saved ? () => void discover() : undefined}
        >
          <p>{error}</p>
        </ErrorBanner>
      )}
      {saved ? (
        <FinishedPanel
          saved={saved}
          launched={launched}
          launchOutput={launchOutput}
          busy={busy !== null}
          error={error}
          copyNotice={copyNotice}
          onRetryLaunch={() => void retryLaunch()}
          onCopy={() => void copy()}
          onTemplates={onTemplates}
          onClose={onClose}
        />
      ) : (
        <>
          {step === 0 && (
            <div className="card catalog-panel">
              <h2>Start with your kit sources</h2>
              <p className="catalog-muted">
                Enable the providers you want to use and choose their versions. Discovery reads kit
                descriptors from those saved references.
              </p>
              <Providers
                disabled={busy !== null}
                onChange={onProvidersChange}
              />
              {providers && (
                <DiscoveryStep
                  providers={providers}
                  hubQuery={hubQuery}
                  onHubQuery={setHubQuery}
                  busy={busy !== null}
                  onDiscover={() => void discover()}
                />
              )}
            </div>
          )}
          {step > 0 && !catalog && (
            <div className="card catalog-panel">
              <p className="catalog-muted">
                No discovery results. Review your kit sources and discover kits again.
              </p>
              <button
                type="button"
                className="primary"
                disabled={busy !== null}
                onClick={() => moveTo(0)}
              >
                Review kit sources
              </button>
            </div>
          )}
          {step > 0 && catalog && (
            <>
              {catalog.warnings.length > 0 && (
                <details className="catalog-warnings" open>
                  <summary>Discovery notes ({catalog.warnings.length})</summary>
                  <ul>
                    {catalog.warnings.map((warning, index) => (
                      <li key={index}>{warning}</li>
                    ))}
                  </ul>
                  <button
                    type="button"
                    className="text-button"
                    disabled={busy !== null}
                    onClick={() => moveTo(0)}
                  >
                    Review sources and retry discovery
                  </button>
                </details>
              )}
              {step === 1 && (
                <AgentStep
                  agents={agents}
                  agent={agent}
                  search={agentSearch}
                  onSearch={setAgentSearch}
                  onSelect={selectAgent}
                  onBack={() => moveTo(0)}
                  onContinue={() => moveTo(2)}
                  onReviewSources={() => moveTo(0)}
                />
              )}
              {step === 2 && agent && (
                <MixinStep
                  agent={agent}
                  compatible={compatible}
                  matchingMixins={matchingMixins}
                  mixinGroups={mixinGroups}
                  mixins={mixins}
                  values={flow.values}
                  search={mixinSearch}
                  onSearch={setMixinSearch}
                  providerFilter={providerFilter}
                  onProviderFilter={setProviderFilter}
                  providerLabel={providerLabel}
                  onToggleMixin={toggleMixin}
                  onRemoveMixin={removeMixin}
                  onArgumentChange={changeArgument}
                  onBack={() => moveTo(1)}
                  onContinue={() => inputValid && moveTo(3)}
                />
              )}
              {step === 3 && request && agent && (
                <ReviewStep
                  agent={agent}
                  mixins={mixins}
                  selectedKits={flow.selectedKits}
                  name={name}
                  onName={setName}
                  workspace={workspace}
                  onWorkspace={setWorkspace}
                  savePath={savePath}
                  onSavePath={setSavePath}
                  busy={busy !== null}
                  request={request}
                  onSubmit={(launch) => void submit(launch)}
                  onBrowse={(target) => void browse(target)}
                  onBack={() => moveTo(2)}
                />
              )}
            </>
          )}
        </>
      )}
    </section>
  );
}
