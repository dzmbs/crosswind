import { defineConfig } from "vocs";
import { sidebar } from "./sidebar/sidebar";

export default defineConfig({
  title: "Crosswind | Google Flights CLI",
  description: "Search Google Flights from your terminal. Built for humans and AI agents.",
  rootDir: ".",
  sidebar,
  theme: {
    colorScheme: "system",
  },
  accentColor: "#0EA5E9",
  logoUrl: "/logo.svg",
  iconUrl: "/logo.svg",
  font: {
    google: "Inter",
  },
  socials: [
    {
      link: "https://github.com/dzmbs/crosswind",
      icon: "github",
    },
  ],
  topNav: [
    { link: "/introduction/getting-started", text: "Docs" },
    { link: "/usage/searching", text: "Usage" },
    { link: "/reference/output", text: "Reference" },
    { link: "/introduction/installation", text: "Install" },
  ],
});
