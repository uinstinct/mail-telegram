import {
  internalMutation,
} from "../_generated/server";
import { v } from "convex/values";
import {nanoid} from 'nanoid'

export const storeMailsInDB = internalMutation({
  args: { content: v.string(), timestamp: v.string(), messageId: v.string(), subject: v.string(), from: v.string()},
  handler: async (ctx, args) => {
    await ctx.db.insert("mails", { hash: nanoid(10), sentOnTelegram: false, isViewable: true, ...args  });
  },
});

export const markMessageAsSent = internalMutation({
  args: {id: v.id('mails')},
  handler: async (ctx, args) => {
    await ctx.db.patch(args.id, { sentOnTelegram: true });
  }
})