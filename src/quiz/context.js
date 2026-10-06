import { createContext, useContext } from "react";

// { quiz, rounds, teams, resolveMedia } for the quiz being shown
export const QuizContext = createContext(null);
export const useQuiz = () => useContext(QuizContext);
