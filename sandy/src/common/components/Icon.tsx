import type { ReactNode } from "react";

const paths = {
  home: <path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1Z" />,
  box: (
    <>
      <path d="m12 3 9 5v9l-9 5-9-5V8Z" />
      <path d="m3 8 9 5 9-5M12 13v9M7.5 5.5l9 5" />
    </>
  ),
  create: (
    <>
      <path d="m12 3 8 4.5v8L12 20l-8-4.5v-8Z" />
      <path d="m4 7.5 8 4.5 8-4.5M12 12v8M20 16v6M17 19h6" />
    </>
  ),
  template: (
    <>
      <path d="M6 3h8l5 5v13H6Z" />
      <path d="M14 3v6h5M9 13h7M9 17h7" />
    </>
  ),
  settings: (
    <>
      <path d="m10 3-.6 2.3-2 .9L5.2 5 3 9l1.8 1.6v2.8L3 15l2.2 4 2.2-1.2 2 .9L10 21h4l.6-2.3 2-.9 2.2 1.2 2.2-4-1.8-1.6v-2.8L21 9l-2.2-4-2.2 1.2-2-.9L14 3Z" />
      <circle cx="12" cy="12" r="3" />
    </>
  ),
  help: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M9.5 9a2.5 2.5 0 0 1 5 0c0 2-2.5 2-2.5 4M12 16h.01" />
    </>
  ),
  user: (
    <>
      <circle cx="12" cy="8" r="3" />
      <path d="M5 21v-2a7 7 0 0 1 14 0v2" />
    </>
  ),
  book: <path d="M12 5C9 3 5 3 2 4v15c3-1 7-1 10 1 3-2 7-2 10-1V4c-3-1-7-1-10 1Zm0 0v15" />,
  arrow: <path d="M5 12h14m-6-6 6 6-6 6" />,
  chevron: <path d="m7 10 5 5 5-5" />,
  search: (
    <>
      <circle cx="10.5" cy="10.5" r="6.5" />
      <path d="m16 16 5 5" />
    </>
  ),
  more: (
    <>
      <circle cx="5" cy="12" r=".8" />
      <circle cx="12" cy="12" r=".8" />
      <circle cx="19" cy="12" r=".8" />
    </>
  ),
  monitor: (
    <>
      <rect x="3" y="3" width="18" height="13" rx="2" />
      <path d="M8 21h8m-4-5v5" />
    </>
  ),
  stop: <circle cx="12" cy="12" r="7" />,
  refresh: (
    <>
      <path d="M20 7v5h-5M4 17v-5h5" />
      <path d="M5.2 8a7 7 0 0 1 11.5-3L20 8M4 16l3.3 3A7 7 0 0 0 18.8 16" />
    </>
  ),
  code: <path d="m7 7-5 5 5 5m10-10 5 5-5 5M14 4l-4 16" />,
  bulb: <path d="M8 16a7 7 0 1 1 8 0v3H8Zm1 6h6M12 1V0M3 5 1 3m20 2 2-2" />,
  check: <path d="m5 12 4 4L19 6" />,
  close: <path d="m6 6 12 12M6 18 18 6" />,
  terminal: (
    <>
      <rect x="2" y="4" width="20" height="16" rx="3" />
      <path d="m6 8 4 4-4 4m7 0h5" />
    </>
  ),
  shield: (
    <>
      <path d="m12 2 9 4v6c0 6-9 10-9 10S3 18 3 12V6Z" />
      <path d="m8 12 3 3 5-6" />
    </>
  ),
  clipboard: (
    <>
      <rect x="5" y="5" width="14" height="17" rx="2" />
      <rect x="9" y="2" width="6" height="5" rx="1" />
      <path d="m9 15 2 2 4-4" />
    </>
  ),
  key: (
    <>
      <circle cx="8" cy="8" r="5" />
      <path d="m12 12 9 9m-5-5 3-3m-6 0 3-3" />
    </>
  ),
  wave: (
    <>
      <g transform="rotate(-18 12 13)" strokeWidth="1.1">
        <path
          d="M7.5 12.5v-6a1.25 1.25 0 0 1 2.5 0v4.3-6.3a1.25 1.25 0 0 1 2.5 0v6.3-5.3a1.25 1.25 0 0 1 2.5 0v6-3.5a1.25 1.25 0 0 1 2.5 0v6.5c0 4-2.5 6.5-6 6.5H11c-2.1 0-3.4-1.1-4.6-2.7l-3.2-4.1a1.45 1.45 0 0 1 2.2-1.8l2.1 2.4Z"
          fill="#ffc857"
          stroke="#d8871c"
        />
        <path d="M7.6 14.5c.5-1.8 2.2-2.7 4-2.4M13 15.2c.6 1.2.4 2.6-.3 3.5" stroke="#e79b2a" />
      </g>
      <path
        d="M18.5 2.8c1.6.8 2.7 2.1 3.1 3.8M20 10l2 .5M3.4 5.8 2.2 4.5M2.5 8.5.8 8"
        stroke="#efaa37"
        strokeWidth="1.4"
      />
    </>
  ),
} satisfies Record<string, ReactNode>;

export type IconName = keyof typeof paths;

export default function Icon({ name, size = 22 }: { name: IconName; size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      className="icon"
    >
      {paths[name]}
    </svg>
  );
}
