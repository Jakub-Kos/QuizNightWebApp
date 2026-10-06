import { teamInitials } from "../quiz/initials";

// A team's photo, or its initials on the team color when it has none (team.initials from resolveTeams
// are distinct within the quiz). Size and rounding come from
// className; textClass sets the size of the initials.
export default function TeamAvatar({ team, className = "", textClass = "text-2xl" }) {
  return (
    <div className={`overflow-hidden shrink-0 flex items-center justify-center ${team.image ? "bg-white/5" : `bg-gradient-to-br ${team.color}`} ${className}`}>
      {team.image ? (
        <img src={team.image} alt={team.name} className="w-full h-full object-cover" />
      ) : (
        <span className={`font-black text-white/90 tracking-tight leading-none drop-shadow-[0_2px_8px_rgba(0,0,0,0.4)] ${textClass}`}>{team.initials || teamInitials(team.name)}</span>
      )}
    </div>
  );
}
