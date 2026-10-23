import Icon from "../../../common/components/Icon";

export function HomeActions({
  onCreate,
  onGoTemplates,
  onLearnMore,
}: {
  onCreate: () => void;
  onGoTemplates: () => void;
  onLearnMore: () => void;
}) {
  return (
    <div className="action-cards">
      <button className="action-card featured" onClick={onCreate}>
        <span className="action-icon">
          <Icon name="create" size={32} />
        </span>
        <span className="action-title">Create sandbox</span>
        <span className="action-arrow">
          <Icon name="arrow" size={19} />
        </span>
      </button>
      <button className="action-card" onClick={onGoTemplates}>
        <span className="action-icon blue">
          <Icon name="template" size={27} />
        </span>
        <span className="action-title">From template</span>
        <span className="action-arrow">
          <Icon name="arrow" size={18} />
        </span>
      </button>
      <button className="action-card" onClick={onLearnMore}>
        <span className="action-icon purple">
          <Icon name="book" size={27} />
        </span>
        <span className="action-title">Learn more</span>
        <span className="action-arrow">
          <Icon name="arrow" size={18} />
        </span>
      </button>
    </div>
  );
}
