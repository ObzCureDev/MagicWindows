import type { Layout } from "@magicwindows/keyboard-visual";
import frAzerty from "../../../layouts/apple-fr-azerty.json";
import usQwerty from "../../../layouts/apple-us-qwerty.json";
import ukQwerty from "../../../layouts/apple-uk-qwerty.json";
import deQwertz from "../../../layouts/apple-de-qwertz.json";
import esQwerty from "../../../layouts/apple-es-qwerty.json";
import itQwerty from "../../../layouts/apple-it-qwerty.json";

export interface LayoutCard {
  id: string;
  displayName: string;
  blurb: string;
  layout: Layout;
}

const all: Array<{ id: string; raw: unknown }> = [
  { id: "apple-fr-azerty", raw: frAzerty },
  { id: "apple-us-qwerty", raw: usQwerty },
  { id: "apple-uk-qwerty", raw: ukQwerty },
  { id: "apple-de-qwertz", raw: deQwertz },
  { id: "apple-es-qwerty", raw: esQwerty },
  { id: "apple-it-qwerty", raw: itQwerty },
];

export const layouts: Record<string, LayoutCard> = Object.fromEntries(
  all.map(({ id, raw }) => {
    const r = raw as { name: { en: string }; description: { en: string } } & Layout;
    return [
      id,
      {
        id,
        displayName: r.name.en,
        blurb: r.description.en,
        layout: r,
      },
    ];
  }),
);

export const layoutIds: string[] = all.map((x) => x.id);
