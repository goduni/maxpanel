import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import ru from "../public/locales/ru/translation.json";

i18n.use(initReactI18next).init({
  lng: "ru",
  fallbackLng: "ru",
  interpolation: {
    // React already escapes JSX output — disabling i18next escaping prevents
    // double-encoding. NEVER use dangerouslySetInnerHTML with translated strings.
    escapeValue: false,
  },
  resources: {
    ru: { translation: ru },
  },
});

export default i18n;
