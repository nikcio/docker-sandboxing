import { useId } from "react";
import type { KitArguments, KitOption } from "../../../lib/kit";
import { argumentValue } from "../../../lib/kit";

export function KitInputs({
  kits,
  values,
  onChange,
  disabled,
}: {
  kits: KitOption[];
  values: KitArguments;
  onChange: (kitId: string, argument: string, value: string) => void;
  disabled: boolean;
}) {
  const id = useId();
  const configurable = kits.filter((kit) => kit.arguments.length > 0);
  if (!configurable.length) {
    return <p className="catalog-help">The selected kits don’t require any inputs.</p>;
  }
  return (
    <div className="catalog-kit-inputs">
      <h3>Kit inputs</h3>
      <p className="catalog-muted">Required inputs are marked. Defaults come from each kit’s metadata.</p>
      {configurable.map((kit, kitIndex) => (
        <fieldset className="catalog-input-group" key={kit.id} disabled={disabled}>
          <legend>{kit.name}</legend>
          <div className="catalog-fields">
            {kit.arguments.map((argument, argumentIndex) => {
              const inputId = `${id}-${kitIndex}-${argumentIndex}`;
              const value = argumentValue(values, kit, argument);
              return (
                <label className="catalog-field" key={argument.name} htmlFor={inputId}>
                  <span>
                    {argument.name}{" "}
                    {argument.required ? (
                      <span className="catalog-required">(required)</span>
                    ) : (
                      <span className="catalog-help">(optional)</span>
                    )}
                  </span>
                  {argument.choices.length ? (
                    <select
                      id={inputId}
                      value={value}
                      required={argument.required}
                      aria-describedby={`${inputId}-help`}
                      onChange={(event) => onChange(kit.id, argument.name, event.target.value)}
                    >
                      {!argument.choices.includes("") && (
                        <option value="">
                          {argument.required ? "Choose a value" : "Use kit default"}
                        </option>
                      )}
                      {argument.choices.map((choice) => (
                        <option key={choice} value={choice}>
                          {choice || "Empty value"}
                        </option>
                      ))}
                    </select>
                  ) : (
                    <input
                      id={inputId}
                      value={value}
                      required={argument.required}
                      placeholder="Value"
                      aria-describedby={`${inputId}-help`}
                      onChange={(event) => onChange(kit.id, argument.name, event.target.value)}
                    />
                  )}
                  <span className="catalog-help" id={`${inputId}-help`}>
                    {argument.description}
                    {argument.default !== null && (
                      <>
                        {" "}
                        Default: <code>{argument.default === "" ? "(empty)" : argument.default}</code>.
                      </>
                    )}
                  </span>
                </label>
              );
            })}
          </div>
        </fieldset>
      ))}
    </div>
  );
}
