import { TRANSLATIONS } from "./data/translations";
import { useHashRoute } from "./hooks/useHashRoute";
import { useUiLanguage } from "./hooks/useUiLanguage";
import { useQuizBundle } from "./hooks/useQuizBundle";
import QuizShow from "./components/QuizShow";
import Library from "./components/Library";
import QuizEditor from "./components/QuizEditor";
import QuizCheck from "./components/QuizCheck";

export default function App() {
  const route = useHashRoute();
  const [lang, setLang] = useUiLanguage();
  const t = TRANSLATIONS[lang] || TRANSLATIONS.en;

  if (route.name === "show" || route.name === "presenter") {
    return <ShowRoute key={route.id} id={route.id} isPresenter={route.name === "presenter"} t={t} />;
  }
  if (route.name === "edit") return <QuizEditor key={route.id} id={route.id} t={t} />;
  if (route.name === "check") return <QuizCheck key={route.id} id={route.id} t={t} />;
  return <Library t={t} lang={lang} onLanguage={setLang} />;
}

function ShowRoute({ id, isPresenter, t }) {
  const state = useQuizBundle(id);

  if (state.status === "ready") return <QuizShow bundle={state.bundle} isPresenter={isPresenter} />;

  return (
    <div className="bg-[#050505] text-white h-screen flex flex-col items-center justify-center gap-6 font-['League_Spartan']">
      {state.status === "loading" ? (
        <>
          <div className="w-16 h-16 border-4 border-t-blue-500 border-white/10 rounded-full animate-spin" />
          <span className="tracking-[0.2em] uppercase text-sm opacity-50">Loading System...</span>
        </>
      ) : (
        <>
          <p className="text-xl text-gray-300">{state.status === "missing" ? t.ed_not_found : state.message}</p>
          <a href="#/" className="px-6 py-3 rounded-xl bg-yellow-500 hover:bg-yellow-400 text-black font-bold">{t.settings_exit}</a>
        </>
      )}
    </div>
  );
}
