import { useState, useEffect, useMemo, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ArrowLeft, ArrowRight, Check, X, Edit3, Hash, Clock, ListOrdered } from "lucide-react";

export default function QuestionScreen({ roundData, mode, onBack, isPresenter, t = {} }) {
    const [qIndex, setQIndex] = useState(-1);
    const [step, setStep] = useState(0);
    const [timer, setTimer] = useState(0);

    const isQuestionsOnly = mode === "questions_only";
    const modeText = isQuestionsOnly ? "Q ONLY MODE" : "REVEAL MODE";

    // --- 2-WAY PRESENTER SYNC ---
    const channel = useMemo(() => new BroadcastChannel('quiz-question-sync'), []);
    const isReceivingRef = useRef(false);

    useEffect(() => {
        channel.postMessage({ type: 'Q_REQUEST', sender: isPresenter ? 'presenter' : 'main' });
    }, [channel, isPresenter]);

    useEffect(() => {
        if (isReceivingRef.current) {
            isReceivingRef.current = false;
            return;
        }
        channel.postMessage({
            type: 'Q_UPDATE',
            sender: isPresenter ? 'presenter' : 'main',
            payload: { qIndex, step }
        });
    }, [qIndex, step, isPresenter, channel]);

    useEffect(() => {
        const handleQSync = (e) => {
            const { type, payload, sender } = e.data;
            const myRole = isPresenter ? 'presenter' : 'main';

            if (sender === myRole) return;

            if (type === 'Q_UPDATE') {
                isReceivingRef.current = true;
                setQIndex(payload.qIndex);
                setStep(payload.step);
            } else if (type === 'Q_REQUEST') {
                channel.postMessage({
                    type: 'Q_UPDATE',
                    sender: myRole,
                    payload: { qIndex, step }
                });
            }
        };
        channel.addEventListener('message', handleQSync);
        return () => channel.removeEventListener('message', handleQSync);
    }, [qIndex, step, isPresenter, channel]);


    // --- DATA PROCESSING (Handles Top5 grouping) ---
    const processedQuestions = useMemo(() => {
        const qs = [];
        let i = 0;

        while (i < roundData.questions.length) {
            const row = roundData.questions[i];
            const type = row[3] ? String(row[3]).trim() : "Written";

            if (type === "Top5") {
                // Start an array with the first answer
                const answers = [row[8]];
                let j = 1;

                // Keep grabbing subsequent rows until we hit a new question text or question type
                while (i + j < roundData.questions.length) {
                    const nextRow = roundData.questions[i + j];
                    if (nextRow[2] || nextRow[3]) break; // A new question starts here
                    answers.push(nextRow[8]);
                    j++;
                }

                qs.push({
                    id: row[1],
                    text: row[2],
                    type: "Top5",
                    options: {},
                    answer: answers, // Array of answers
                    source: row[9] ? `source/${row[9]}` : null
                });

                i += j; // Skip the rows we just absorbed
            } else {
                qs.push({
                    id: row[1],
                    text: row[2],
                    type: type,
                    options: { A: row[4], B: row[5], C: row[6], D: row[7] },
                    answer: row[8],
                    source: row[9] ? `source/${row[9]}` : null
                });
                i++;
            }
        }
        return qs;
    }, [roundData.questions]);

    const question = qIndex >= 0 ? processedQuestions[qIndex] : null;

    const getExpectedAnswerText = (type) => {
        switch(type) {
            case "Written": return t.type_written || "WRITTEN ANSWER";
            case "Numeric": return t.type_numeric || "NUMERIC ANSWER";
            case "Top5": return "TOP 5 LIST";
            default: return type.toUpperCase();
        }
    };

    const getStepsForType = (type) => {
        const baseSteps = (type === "PImage" || type === "PVideo") ? 4 : 3;
        if (isQuestionsOnly) return baseSteps - 1;
        return baseSteps;
    };

    // --- TIMER LOGIC ---
    useEffect(() => {
        let interval;
        const isRunning = isQuestionsOnly && qIndex >= 0 && step > 0;
        if (isRunning) {
            interval = setInterval(() => setTimer(t => t + 1), 1000);
        }
        return () => clearInterval(interval);
    }, [step, qIndex, isQuestionsOnly]);

    useEffect(() => { setTimer(0); }, [qIndex]);

    // --- CONTROLS ---
    useEffect(() => {
        const handleKeyDown = (e) => {
            if (e.key === "ArrowRight") next();
            if (e.key === "ArrowLeft") prev();
            if (e.key === "Escape") onBack();
        };
        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [step, qIndex, isQuestionsOnly, isPresenter]);

    const next = () => {
        if (qIndex === -1) { setQIndex(0); setStep(0); return; }
        const maxSteps = getStepsForType(question.type);
        if (step < maxSteps - 1) {
            setStep(prev => prev + 1);
        } else {
            if (qIndex < processedQuestions.length - 1) {
                setQIndex(prev => prev + 1);
                setStep(0);
            } else { onBack(); }
        }
    };

    const prev = () => {
        if (step > 0) {
            setStep(prev => prev - 1);
        } else {
            if (qIndex > 0) {
                setQIndex(prev => prev - 1);
                const prevType = processedQuestions[qIndex-1].type;
                setStep(getStepsForType(prevType) - 1);
            } else { setQIndex(-1); }
        }
    };

    const getFontSize = (text) => {
        if (!text) return "text-5xl";
        if (text.length < 30) return "text-7xl";
        if (text.length < 80) return "text-6xl";
        return "text-5xl";
    };

    // ==========================================
    // PRESENTER DASHBOARD UI
    // ==========================================
    if (isPresenter) {
        return (
            <div className="h-screen w-full bg-[#0a0a0a] text-white p-10 font-sans flex flex-col">
                {/* Header */}
                <div className="flex justify-between items-center mb-8 border-b border-white/10 pb-6">
                    <div>
                        <h1 className="text-3xl font-black text-yellow-500 tracking-widest uppercase">Host Dashboard</h1>
                        <div className="flex items-center gap-3 mt-2">
                            <span className="text-white bg-white/10 px-3 py-1 rounded text-sm font-bold">{roundData.title}</span>
                            <span className="text-blue-400 font-mono text-sm">{modeText}</span>
                        </div>
                    </div>
                    <div className="flex gap-4">
                        <button onClick={prev} className="px-8 py-4 bg-white/10 hover:bg-white/20 text-white font-bold rounded-xl transition-all">
                            &lt; PREVIOUS
                        </button>
                        <button onClick={next} className="px-12 py-4 bg-blue-600 hover:bg-blue-500 text-white font-black rounded-xl transition-all shadow-[0_0_20px_rgba(59,130,246,0.4)]">
                            NEXT ACTION &gt;
                        </button>
                        <button onClick={onBack} className="px-6 py-4 bg-red-900/50 hover:bg-red-800/80 text-white font-bold rounded-xl transition-all ml-8">
                            EXIT ROUND
                        </button>
                    </div>
                </div>

                {/* Main Content Area */}
                <div className="grid grid-cols-2 gap-10 flex-1">

                    {/* Left Col: WHAT IS ON THE SCREEN NOW */}
                    <div className="bg-black/50 p-8 rounded-[2rem] border border-white/10 flex flex-col">
                      <span className="text-blue-500 font-bold tracking-widest uppercase mb-6 text-sm flex items-center gap-2">
                          <span className="w-2 h-2 rounded-full bg-blue-500 animate-pulse"/> LIVE ON MAIN SCREEN
                      </span>

                        {qIndex === -1 ? (
                            <div className="flex-1 flex items-center justify-center text-4xl font-black text-white/30 uppercase text-center">
                                Round Intro Screen
                            </div>
                        ) : (
                            <div className="flex flex-col h-full">
                                <div className="flex justify-between items-end mb-6 border-b border-white/10 pb-4">
                                    <h2 className="text-4xl font-black text-white">Question {qIndex + 1} / {processedQuestions.length}</h2>
                                    <span className="text-white/40 font-mono">Step {step} / {getStepsForType(question.type) - 1}</span>
                                </div>

                                <p className="text-2xl text-white/90 leading-relaxed font-medium mb-8">
                                    {question.text}
                                </p>

                                {/* Display Options if applicable */}
                                {(question.type === "ABCD" || question.type === "Yes/No") && (
                                    <div className="grid grid-cols-2 gap-4 mb-8">
                                        {question.type === "ABCD" ? ["A", "B", "C", "D"].map(opt => (
                                            <div key={opt} className={`p-4 rounded-xl border font-bold ${String(question.answer).trim().toUpperCase() === opt ? 'bg-green-600/20 border-green-500 text-green-400' : 'bg-white/5 border-white/10 text-white/50'}`}>
                                                {opt}: {question.options[opt]}
                                            </div>
                                        )) : ["Yes", "No"].map(opt => (
                                            <div key={opt} className={`p-4 rounded-xl border font-bold ${String(question.answer).trim().toLowerCase() === opt.toLowerCase() ? 'bg-green-600/20 border-green-500 text-green-400' : 'bg-white/5 border-white/10 text-white/50'}`}>
                                                {opt}
                                            </div>
                                        ))}
                                    </div>
                                )}

                                {/* Highlight the Answer securely for the Host */}
                                <div className="mt-auto bg-green-900/30 border border-green-500/50 p-6 rounded-2xl flex flex-col gap-2">
                                    <span className="text-green-500/80 font-bold uppercase tracking-widest text-xs">Correct Answer</span>
                                    {question.type === "Top5" ? (
                                        <div className="flex flex-col gap-2 mt-2">
                                            {question.answer.map((ans, i) => (
                                                <div key={i} className="flex gap-3 items-center">
                                                    <span className="text-green-500/50 font-bold font-mono text-sm">#{i+1}</span>
                                                    <span className="text-2xl font-black text-green-400">{ans}</span>
                                                </div>
                                            ))}
                                        </div>
                                    ) : (
                                        <span className="text-4xl font-black text-green-400">{question.answer}</span>
                                    )}
                                </div>

                                {/* Mirror Timer */}
                                {isQuestionsOnly && (
                                    <div className="mt-6 flex items-center gap-3 text-yellow-500 font-mono text-xl bg-yellow-500/10 p-4 rounded-xl border border-yellow-500/20 w-fit shadow-inner">
                                        <Clock size={24} /> {Math.floor(timer / 60)}:{(timer % 60).toString().padStart(2, '0')}
                                    </div>
                                )}
                            </div>
                        )}
                    </div>

                    {/* Right Col: UPCOMING INFO / NOTES */}
                    <div className="bg-white/5 p-8 rounded-[2rem] border border-white/5 flex flex-col relative overflow-hidden">
                        <span className="text-gray-400 font-bold tracking-widest uppercase mb-6 text-sm">Action upon clicking "Next"</span>

                        <div className="flex-1 flex flex-col justify-center items-center text-center px-10">
                            <ArrowRight size={80} className="text-white/5 mb-8" />
                            <h3 className="text-3xl font-bold text-white/60">
                                {qIndex === -1 ? "Reveal Big Question 1 Number" :
                                    step < getStepsForType(question?.type) - 1 ? "Advance animation (Show text/media/answer)" :
                                        qIndex < processedQuestions.length - 1 ? `Proceed to Question ${qIndex + 2}` :
                                            "Finish Round and Return to Dashboard"}
                            </h3>
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    // ==========================================
    // NORMAL MAIN SCREEN UI
    // ==========================================
    const isPreMedia = (question?.type === "PImage" || question?.type === "PVideo");
    const showBigNum = step === 0;
    const showPreMedia = isPreMedia && step === 1;
    const showQuestion = !showBigNum && !showPreMedia;

    const isRevealStep = (isPreMedia && step === 3) || (!isPreMedia && step === 2);
    const showAnswer = !isQuestionsOnly && isRevealStep;

    // Update options check to include our custom Top5 render
    const showOptions = showQuestion && (question?.type === "ABCD" || question?.type === "Yes/No" || question?.type === "Sort" || question?.type === "Top5");

    // 1. ROUND INTRO
    if (qIndex === -1) {
        const displayTitle = `${t?.round || "Round"} ${roundData.number}`;
        const parts = (roundData.title || "").split("-");
        const themeName = parts.length > 1 ? parts.slice(1).join("-").trim() : roundData.title;

        return (
            <div onClick={next} className="h-screen w-full bg-[#050505] flex flex-col items-center justify-center font-['League_Spartan'] cursor-pointer relative overflow-hidden">
                <div className="absolute inset-0 bg-gradient-to-b from-[#111] to-black z-0" />
                <motion.div initial={{opacity:0, scale: 0.9}} animate={{opacity:1, scale: 1}} className="text-center z-10 flex flex-col items-center max-w-6xl px-8">
                    <h2 className="text-blue-500 font-bold tracking-[0.5em] text-4xl mb-6 uppercase drop-shadow-lg">{displayTitle}</h2>
                    <div className="h-2 w-40 bg-blue-500 mx-auto mb-12 rounded-full shadow-[0_0_20px_rgba(59,130,246,0.8)]" />
                    <h1 className="text-[120px] font-black text-white uppercase tracking-tight drop-shadow-2xl leading-none text-center">
                        {themeName}
                    </h1>
                </motion.div>
            </div>
        );
    }

    return (
        <div className="h-screen w-full bg-[#050505] relative overflow-hidden flex flex-col font-['League_Spartan']">
            <div className="absolute inset-0 bg-gradient-to-b from-[#111] to-black z-0" />

            {/* BIG NUMBER */}
            <div className="relative h-[25vh] w-full flex items-center justify-center z-20 pointer-events-none">
                <motion.div
                    initial={false}
                    animate={showBigNum ? { scale: 1, y: "30vh" } : { scale: 0.4, y: 0 }}
                    transition={{ duration: 0.8, type: "spring", bounce: 0.2 }}
                    className="flex flex-col items-center"
                >
                <span className={`font-black text-white text-5xl uppercase tracking-widest transition-colors duration-500 ${showBigNum ? "opacity-100 text-blue-500" : "opacity-50 text-gray-500"}`}>
                   {t.question || "QUESTION"}
                </span>
                    <span className="font-black text-[250px] text-white leading-none drop-shadow-[0_0_50px_rgba(255,255,255,0.2)]">
                    {question.id}
                </span>
                </motion.div>
            </div>

            {/* CONTENT */}
            <div className="flex-1 flex flex-col items-center justify-start relative z-10 px-16 pt-4">
                <AnimatePresence mode="wait">
                    {showPreMedia && (
                        <motion.div key="pre-media" initial={{ opacity: 0, scale: 0.95 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0, scale: 1.05 }} className="w-full h-[65vh] flex items-center justify-center">
                            {question.type === "PVideo" ? (
                                <video src={question.source} controls autoPlay className="max-h-full max-w-full rounded-2xl shadow-2xl border border-white/10" />
                            ) : (
                                <img src={question.source} alt="Visual" className="max-h-full max-w-full rounded-2xl shadow-2xl border border-white/10" />
                            )}
                        </motion.div>
                    )}

                    {showQuestion && (
                        <motion.div key="question-content" initial={{ opacity: 0, y: 50 }} animate={{ opacity: 1, y: 0 }} className="w-full max-w-[1600px] flex flex-col items-center text-center">
                            <h2 className={`${getFontSize(question.text)} font-bold text-white leading-tight mb-8 drop-shadow-xl`}>
                                {question.text}
                            </h2>

                            {question.source && !isPreMedia && (
                                <div className="mb-8 max-h-[40vh] relative">
                                    {question.type.includes("Video") ? (
                                        <video src={question.source} controls className="max-h-[40vh] rounded-2xl border border-white/20" />
                                    ) : question.type === "Audio" ? (
                                        <div className="bg-white/10 p-6 rounded-full border border-white/10 flex items-center justify-center w-[500px]">
                                            <audio controls src={question.source} className="w-full" />
                                        </div>
                                    ) : (
                                        <img src={question.source} className="max-h-[40vh] rounded-2xl border border-white/20" />
                                    )}
                                </div>
                            )}

                            <div className="w-full flex-1 flex items-center justify-center min-h-[200px]">
                                {/* ABCD Layout */}
                                {question.type === "ABCD" && (
                                    <div className="grid grid-cols-2 gap-6 w-full max-w-[1400px]">
                                        {["A", "B", "C", "D"].map((opt) => {
                                            const isCorrect = String(question.answer).trim().toUpperCase() === opt;
                                            let style = "bg-white/5 border-white/10 text-gray-300";
                                            if (showAnswer) {
                                                if (isCorrect) style = "bg-green-600 border-green-400 shadow-[0_0_40px_rgba(22,163,74,0.6)] scale-105 z-10 text-white";
                                                else style = "opacity-20 grayscale border-transparent";
                                            }
                                            return (
                                                <div key={opt} className={`p-6 rounded-3xl border flex items-center gap-6 transition-all duration-700 ${style}`}>
                                                    <div className="w-16 h-16 rounded-full flex items-center justify-center font-black text-2xl bg-black/30 border border-white/10 shrink-0">{opt}</div>
                                                    <span className="font-bold text-4xl text-left leading-tight">{question.options[opt]}</span>
                                                </div>
                                            )
                                        })}
                                    </div>
                                )}

                                {/* Yes/No Layout */}
                                {question.type === "Yes/No" && (
                                    <div className="grid grid-cols-2 gap-12 w-full max-w-3xl">
                                        {["Yes", "No"].map(val => {
                                            const isCorrect = String(question.answer).trim().toLowerCase() === val.toLowerCase();
                                            let style = "bg-white/5 border-white/10";
                                            if(showAnswer) {
                                                if(isCorrect) style = "bg-green-600 border-green-400 scale-110 shadow-[0_0_40px_rgba(22,163,74,0.6)]";
                                                else style = "opacity-20";
                                            }
                                            return (
                                                <div key={val} className={`h-48 rounded-[3rem] border flex flex-col items-center justify-center gap-4 transition-all duration-500 ${style}`}>
                                                    {val === "Yes" ? <Check size={64}/> : <X size={64}/>}
                                                    <span className="text-5xl font-black uppercase">{val}</span>
                                                </div>
                                            )
                                        })}
                                    </div>
                                )}

                                {/* Top5 Layout */}
                                {question.type === "Top5" && (
                                    <div className="flex flex-col gap-4 w-full max-w-4xl">
                                        {question.answer.map((ans, idx) => (
                                            <div key={idx} className={`px-8 py-5 rounded-2xl border flex items-center gap-6 transition-all duration-700 ${showAnswer ? 'bg-gradient-to-r from-blue-900/60 to-blue-800/40 border-blue-500 shadow-[0_0_20px_rgba(59,130,246,0.4)]' : 'bg-white/5 border-white/10'}`}>
                                                <div className="w-16 h-16 rounded-xl bg-black/50 border border-white/10 flex items-center justify-center font-black text-2xl text-white/50 shrink-0 shadow-inner">
                                                    #{idx + 1}
                                                </div>
                                                <span className={`font-bold text-5xl text-left tracking-tight transition-all duration-1000 ${showAnswer ? 'text-white' : 'text-transparent bg-white/10 rounded-lg select-none blur-sm'}`}>
                                                {showAnswer ? ans : "???????????????"}
                                            </span>
                                            </div>
                                        ))}
                                    </div>
                                )}

                                {/* Written / Numeric Answers Layout */}
                                {!showOptions && (
                                    <div className="flex flex-col items-center">
                                        {showAnswer ? (
                                            <motion.div initial={{ rotateX: -90, opacity: 0 }} animate={{ rotateX: 0, opacity: 1 }} transition={{ type: "spring", bounce: 0.4 }} className="px-20 py-10 bg-gradient-to-br from-green-500 to-green-600 rounded-[3rem] text-black font-black text-7xl shadow-[0_0_80px_rgba(34,197,94,0.6)] uppercase tracking-tight">
                                                {String(question.answer) || "Answer"}
                                            </motion.div>
                                        ) : (
                                            <div className="flex items-center gap-6 px-16 py-8 rounded-[3rem] bg-white/5 border border-white/10 text-white/50 animate-pulse shadow-inner">
                                                {question.type === "Top5" ? <ListOrdered size={40} /> : question.type === "Numeric" ? <Hash size={40} /> : <Edit3 size={40} />}
                                                <span className="font-black text-4xl tracking-[0.2em] uppercase">{getExpectedAnswerText(question.type)}</span>
                                            </div>
                                        )}
                                    </div>
                                )}
                            </div>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>

            {/* --- FOOTER --- */}
            <div className="h-[10vh] border-t border-white/5 bg-black/50 backdrop-blur-md flex items-center justify-between px-10 relative z-30">
                <div className="flex items-center gap-4 text-white/40 font-mono text-xs uppercase tracking-widest font-bold">
                    <span className="bg-white/10 px-3 py-1.5 rounded-md text-white">{t?.round || "Round"} {roundData.number}</span>
                    <span>Q {qIndex + 1} / {processedQuestions.length}</span>
                    <span className="text-blue-500">{modeText}</span>
                </div>

                {isQuestionsOnly && (
                    <div className="absolute left-1/2 -translate-x-1/2 flex items-center gap-3 text-white font-mono text-3xl font-bold bg-white/5 px-8 py-2.5 rounded-full border border-white/10 shadow-lg transition-colors">
                        <Clock size={24} className={timer > 0 ? "text-yellow-500 animate-pulse" : "text-gray-500"} />
                        <span>{Math.floor(timer / 60)}:{(timer % 60).toString().padStart(2, '0')}</span>
                    </div>
                )}

                <div className="flex items-center gap-3 text-white/30 text-xs uppercase font-bold tracking-wider">
                    <button onClick={prev} className="p-4 bg-white/5 rounded-full hover:bg-white/10 hover:text-white transition-all hover:scale-110"><ArrowLeft size={20}/></button>
                    <button onClick={next} className="p-4 bg-white/10 border border-white/20 rounded-full hover:bg-blue-500 hover:border-blue-500 hover:shadow-[0_0_15px_rgba(59,130,246,0.5)] transition-all hover:scale-110 text-white"><ArrowRight size={20}/></button>
                </div>
            </div>
        </div>
    );
}