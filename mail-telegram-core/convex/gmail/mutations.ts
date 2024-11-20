import {
  internalMutation,
} from "../_generated/server";
import { v } from "convex/values";
import {nanoid} from 'nanoid'

export const storeMailsInDB = internalMutation({
  args: { content: v.string()},
  handler: async (ctx, args) => {
    await ctx.db.insert("mails", { content: args.content, uid: nanoid(10) });
  },
});
