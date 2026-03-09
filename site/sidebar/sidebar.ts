import type { Sidebar } from "vocs";

const docs = [
  {
    text: "Introduction",
    items: [
      { text: "Getting Started", link: "/introduction/getting-started" },
      { text: "Installation", link: "/introduction/installation" },
    ],
  },
  {
    text: "Usage",
    items: [
      { text: "Searching Flights", link: "/usage/searching" },
      { text: "Date Formats", link: "/usage/dates" },
      { text: "Filters", link: "/usage/filters" },
      { text: "Multi-Destination", link: "/usage/multi-destination" },
    ],
  },
  {
    text: "Reference",
    items: [
      { text: "Output Modes", link: "/reference/output" },
      { text: "Exit Codes", link: "/reference/exit-codes" },
      { text: "Agent Integration", link: "/reference/agent-integration" },
      { text: "Limitations", link: "/reference/limitations" },
    ],
  },
];

export const sidebar: Sidebar = {
  "/": [],
  "/introduction": docs,
  "/usage": docs,
  "/reference": docs,
};
