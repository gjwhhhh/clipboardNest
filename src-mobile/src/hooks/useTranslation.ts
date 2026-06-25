import { useTranslation as useI18nextTranslation } from "react-i18next";

export function useTranslation() {
  const { t, i18n } = useI18nextTranslation();

  const changeLanguage = (lang: string) => {
    i18n.changeLanguage(lang);
    localStorage.setItem("language", lang);
  };

  const currentLanguage = i18n.language;

  return {
    t,
    changeLanguage,
    currentLanguage,
  };
}
