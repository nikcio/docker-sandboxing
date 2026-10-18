import Icon from "../../../common/components/Icon";

export function HomeActions({
  onCreate,
  onGoTemplates,
}: {
  onCreate: () => void;
  onGoTemplates: () => void;
}) {
  return (
    <div className="action-cards">
      <button className="action-card featured" onClick={onCreate}>
        <span className="action-icon">
          <Icon name="create" size={32} />
        </span>
        <span className="action-title">Create sandbox</span>
        <span className="action-desc">Start with a project and template.</span>
        <span className="action-arrow">
          <Icon name="arrow" size={19} />
        </span>
      </button>
      <button className="action-card" onClick={onGoTemplates}>
        <span className="action-icon blue">
          <Icon name="template" size={27} />
        </span>
        <span className="action-title">From template</span>
        <span className="action-desc">Use a pre-configured environment.</span>
        <span className="action-arrow">
          <Icon name="arrow" size={18} />
        </span>
      </button>
      <a className="action-card" href="https://docs.docker.com/ai/sandboxes/" target="_blank" rel="noreferrer">
        <span className="action-icon purple">
          <Icon name="book" size={27} />
        </span>
        <span className="action-title">Learn more</span>
        <span className="action-desc">View quick start guide and tips.</span>
        <span className="action-arrow">
          <Icon name="arrow" size={18} />
        </span>
      </a>
    </div>
  );
}
