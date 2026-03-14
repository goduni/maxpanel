import { create } from "zustand";
import { persist } from "zustand/middleware";
import i18n from "@/i18n";

interface LocaleState {
  locale: string;
  setLocale: (locale: string) => void;
}

export const useLocaleStore = create<LocaleState>()(
  persist(
    (set) => ({
      locale: "ru",
      setLocale: (locale) => {
        i18n.changeLanguage(locale);
        set({ locale });
      },
    }),
    {
      name: "maxpanel-locale",
      onRehydrateStorage: () => (state) => {
        // Sync i18n language with persisted locale after hydration
        if (state?.locale) {
          i18n.changeLanguage(state.locale);
        }
      },
    },
  ),
);
