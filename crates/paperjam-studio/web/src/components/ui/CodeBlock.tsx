interface CodeBlockProps {
  children: string;
  className?: string;
}

export default function CodeBlock({ children, className = "" }: CodeBlockProps) {
  return (
    <pre className={`code-block${className ? ` ${className}` : ""}`}>
      {children || "No data"}
    </pre>
  );
}
