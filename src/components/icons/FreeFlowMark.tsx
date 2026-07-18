const FreeFlowMark = ({
  width,
  height,
}: {
  width?: number | string;
  height?: number | string;
}) => (
  <svg
    width={width || 126}
    height={height || 126}
    viewBox="0 0 126 126"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect width="126" height="126" rx="28" className="fill-bg" />
    <path
      d="M20 52c12 0 12-18 24-18s12 34 24 34 12-22 24-22 12 12 16 12"
      className="stroke-logo-primary"
      strokeWidth="9"
      strokeLinecap="round"
    />
    <path
      d="M20 70c12 0 12-18 24-18s12 34 24 34 12-22 24-22 12 12 16 12"
      stroke="#68f0b7"
      strokeWidth="7"
      strokeLinecap="round"
    />
  </svg>
);

export default FreeFlowMark;
