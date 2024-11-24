import { cronJobs } from "convex/server";
import { internal } from "./_generated/api";

const crons = cronJobs();

crons.daily(
  "fetch gmails cron",
  { hourUTC: 4, minuteUTC: 30 },
  internal.gmail.actions.fetchGmails,
);

export default crons;