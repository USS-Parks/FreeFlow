import FreeFlowMark from "./FreeFlowMark";

const FreeFlowWordmark = ({
  width = 180,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => (
  <div className={`flex items-center gap-2 ${className ?? ""}`}>
    <FreeFlowMark
      width={Math.round(width * 0.28)}
      height={Math.round(width * 0.28)}
    />
    <svg
      width={Math.round(width * 0.72)}
      height={Math.round(width * 0.28)}
      viewBox="0 0 180 70"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M8 58V12h36M8 34h28"
        className="stroke-text"
        strokeWidth="9"
        strokeLinecap="round"
      />
      <path
        d="M58 48c12 0 12-22 24-22s12 30 24 30 12-24 24-24 12 16 24 16 12-12 18-12"
        className="stroke-logo-primary"
        strokeWidth="8"
        strokeLinecap="round"
      />
    </svg>
  </div>
);

export default FreeFlowWordmark;
