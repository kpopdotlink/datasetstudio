import { Moon, Sun, Globe } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useThemeStore } from "../../stores/themeStore";

const languages = [
  { code: "en", label: "English" },
  { code: "ko", label: "한국어" },
];

export default function Header() {
  const { t, i18n } = useTranslation();
  const { theme, toggleTheme } = useThemeStore();

  const currentLang = languages.find((l) => l.code === i18n.language) || languages[0];

  const handleLanguageChange = (langCode: string) => {
    i18n.changeLanguage(langCode);
  };

  return (
    <header className="flex h-12 items-center justify-between border-b border-border px-4">
      <div className="flex items-center gap-2">
        <span className="text-sm text-muted-foreground">{t("project.title")}</span>
      </div>
      <div className="flex items-center gap-2">
        {/* 언어 선택 */}
        <div className="relative group">
          <button
            className="flex items-center gap-1.5 rounded-md px-2 py-1.5 text-sm hover:bg-secondary"
            title={t("settings.language")}
          >
            <Globe className="h-4 w-4" />
            <span className="text-xs">{currentLang.label}</span>
          </button>
          <div className="absolute right-0 top-full mt-1 hidden w-32 rounded-md border border-border bg-background shadow-lg group-hover:block z-50">
            {languages.map((lang) => (
              <button
                key={lang.code}
                onClick={() => handleLanguageChange(lang.code)}
                className={`w-full px-3 py-2 text-left text-sm hover:bg-secondary ${
                  i18n.language === lang.code ? "bg-primary/10 font-medium" : ""
                }`}
              >
                {lang.label}
              </button>
            ))}
          </div>
        </div>

        {/* 테마 토글 */}
        <button
          onClick={toggleTheme}
          className="rounded-md p-2 hover:bg-secondary"
          title={theme === "light" ? t("settings.dark") : t("settings.light")}
        >
          {theme === "light" ? (
            <Moon className="h-4 w-4" />
          ) : (
            <Sun className="h-4 w-4" />
          )}
        </button>
      </div>
    </header>
  );
}
