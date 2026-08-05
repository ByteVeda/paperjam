import type { ReactNode, ButtonHTMLAttributes } from "react";

type ButtonVariant = "default" | "primary" | "ghost";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  icon?: ReactNode;
  size?: "default" | "sm";
}

const variantClass: Record<ButtonVariant, string> = {
  default: "btn",
  primary: "btn btn-primary",
  ghost: "btn btn-ghost",
};

export default function Button({
  variant = "default",
  icon,
  size = "default",
  children,
  className = "",
  ...rest
}: ButtonProps) {
  const base = variantClass[variant];
  const sizeClass = size === "sm" ? " btn-sm" : "";
  const cls = `${base}${sizeClass}${className ? ` ${className}` : ""}`;

  return (
    <button className={cls} {...rest}>
      {icon}
      {children}
    </button>
  );
}
