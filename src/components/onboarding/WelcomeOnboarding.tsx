import { useTranslation } from "react-i18next";
import { CloudOff, HardDrive, ShieldCheck } from "lucide-react";
import FreeFlowWordmark from "../icons/FreeFlowWordmark";

interface WelcomeOnboardingProps {
  onContinue: () => void;
}

const WelcomeOnboarding: React.FC<WelcomeOnboardingProps> = ({
  onContinue,
}) => {
  const { t } = useTranslation();
  const promises = [
    { icon: HardDrive, text: t("onboarding.welcome.local") },
    { icon: CloudOff, text: t("onboarding.welcome.noAccount") },
    { icon: ShieldCheck, text: t("onboarding.welcome.network") },
  ];

  return (
    <main className="h-screen w-screen flex items-center justify-center p-6">
      <div className="max-w-lg w-full flex flex-col items-center gap-7 text-center">
        <FreeFlowWordmark width={220} />
        <div className="space-y-2">
          <h1 className="text-2xl font-semibold text-text">
            {t("onboarding.welcome.title")}
          </h1>
          <p className="text-text/70">{t("onboarding.welcome.description")}</p>
        </div>
        <ul className="w-full grid gap-3 text-left">
          {promises.map(({ icon: Icon, text }) => (
            <li
              key={text}
              className="flex items-start gap-3 rounded-lg border border-mid-gray/20 bg-white/5 p-4"
            >
              <Icon className="w-5 h-5 mt-0.5 shrink-0 text-logo-primary" />
              <span className="text-sm text-text/80">{text}</span>
            </li>
          ))}
        </ul>
        <button
          type="button"
          onClick={onContinue}
          className="w-full rounded-lg bg-logo-primary px-5 py-3 font-medium text-white hover:bg-logo-primary/90"
        >
          {t("onboarding.welcome.continue")}
        </button>
      </div>
    </main>
  );
};

export default WelcomeOnboarding;
