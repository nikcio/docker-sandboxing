import Icon from "./Icon";

export function Avatar({ name }: { name: string }) {
  let hash = 0;
  for (const character of name) {
    hash = (hash * 31 + character.charCodeAt(0)) % 997;
  }
  return (
    <span className={`sbx-avatar palette-${hash % 5}`}>
      <Icon name={hash % 3 === 0 ? "code" : "box"} size={27} />
    </span>
  );
}
