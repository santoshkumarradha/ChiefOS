declare module "react" {
  export type Key = string | number;
  export type ReactText = string | number;
  export type ReactNode =
    | ReactElement
    | ReactText
    | boolean
    | null
    | undefined
    | readonly ReactNode[];

  export interface ReactElement<P = unknown, T = unknown> {
    type: T;
    props: P;
    key: Key | null;
  }

  export type ComponentType<P = Record<string, never>> = (props: P) => ReactElement | null;
  export type ElementType<P = Record<string, never>> = keyof JSX.IntrinsicElements | ComponentType<P>;

  export interface CSSProperties {
    [key: string]: string | number | undefined;
  }

  export interface DOMAttributes<T> {
    onClick?: (event: unknown) => void;
    onKeyDown?: (event: unknown) => void;
    onMouseDown?: (event: unknown) => void;
    onMouseUp?: (event: unknown) => void;
    onMouseLeave?: (event: unknown) => void;
    onTouchStart?: (event: unknown) => void;
    onTouchEnd?: (event: unknown) => void;
    onChange?: (event: unknown) => void;
  }

  export interface HTMLAttributes<T> extends DOMAttributes<T> {
    id?: string;
    role?: string;
    title?: string;
    tabIndex?: number;
    className?: string;
    children?: ReactNode;
    style?: CSSProperties;
    hidden?: boolean;
    "aria-label"?: string;
    "aria-selected"?: boolean;
    "aria-valuemin"?: number;
    "aria-valuemax"?: number;
    "aria-valuenow"?: number;
    "aria-valuetext"?: string;
    "aria-current"?: boolean | "page" | "step" | "location" | "date" | "time";
    "data-chief"?: string;
    [dataAttribute: `data-${string}`]: string | number | boolean | undefined;
  }

  export interface ButtonHTMLAttributes<T> extends HTMLAttributes<T> {
    type?: "button" | "submit" | "reset";
    disabled?: boolean;
  }

  export interface InputHTMLAttributes<T> extends HTMLAttributes<T> {
    type?: string;
    value?: string | number | readonly string[];
    placeholder?: string;
    autoFocus?: boolean;
  }

  export interface AriaAttributes {
    "aria-label"?: string;
  }

  export function createElement<P>(
    type: ElementType<P>,
    props: (P & { key?: Key | null }) | null,
    ...children: ReactNode[]
  ): ReactElement<P> | null;
}

declare namespace JSX {
  interface IntrinsicElements {
    [elementName: string]: Record<string, unknown>;
  }
}
