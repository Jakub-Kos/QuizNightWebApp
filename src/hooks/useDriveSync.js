import { useState, useEffect } from "react";
import { getSyncState, onSyncState } from "../quiz/driveConfig";

// The Drive sync state of a quiz (see getSyncState), updated as it changes
export function useDriveSync(quizId) {
  const [state, setState] = useState(() => getSyncState(quizId));
  useEffect(() => onSyncState((id, next) => id === quizId && setState(next)), [quizId]);
  return state;
}
