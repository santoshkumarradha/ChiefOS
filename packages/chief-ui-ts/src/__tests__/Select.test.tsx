import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@testing-library/react";
import { Select, type SelectProps } from "../primitives.js";

describe("Select", () => {
  const defaultProps: SelectProps = {
    options: [
      { id: "fast", label: "Fast" },
      { id: "deep", label: "Deep" },
    ],
    selected: "fast",
    onSelect: vi.fn(),
    placeholder: "Choose a tier",
  };

  it("renders select with options", () => {
    const { getByDisplayValue } = render(<Select {...defaultProps} />);
    expect(getByDisplayValue("Choose a tier")).toBeTruthy();
  });

  it("displays selected value", () => {
    const { container } = render(<Select {...defaultProps} />);
    const select = container.querySelector("select") as HTMLSelectElement;
    expect(select.value).toBe("fast");
  });

  it("calls onSelect when option changes", () => {
    const onSelect = vi.fn();
    const { container } = render(<Select {...defaultProps} onSelect={onSelect} />);
    const select = container.querySelector("select") as HTMLSelectElement;

    fireEvent.change(select, { target: { value: "deep" } });
    expect(onSelect).toHaveBeenCalledWith("deep");
  });

  it("disables options marked as disabled", () => {
    const props: SelectProps = {
      ...defaultProps,
      options: [
        { id: "fast", label: "Fast" },
        { id: "deep", label: "Deep", disabled: true },
      ],
    };
    const { container } = render(<Select {...props} />);
    const options = Array.from(container.querySelectorAll("option"));
    const deepOption = options.find((o) => o.textContent === "Deep") as HTMLOptionElement;
    expect(deepOption.disabled).toBe(true);
  });

  it("respects disabled prop on select element", () => {
    const { container } = render(<Select {...defaultProps} disabled={true} />);
    const select = container.querySelector("select") as HTMLSelectElement;
    expect(select.disabled).toBe(true);
  });

  it("renders placeholder option when selected is empty", () => {
    const { container } = render(
      <Select {...defaultProps} selected={""} />
    );
    const options = Array.from(container.querySelectorAll("option"));
    const placeholderOption = options[0];
    expect(placeholderOption.textContent).toBe("Choose a tier");
    expect((placeholderOption as HTMLOptionElement).disabled).toBe(true);
  });
});
