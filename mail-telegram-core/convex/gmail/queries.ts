import { internalQuery } from "../_generated/server";
import { v } from "convex/values";

export const getMails = internalQuery({
  handler: async (ctx) => {
    const result = await ctx.db
      .query("mails")
      .filter((q) =>
        q.and(
          q.eq(q.field("sentOnTelegram"), false),
          q.eq(q.field("isViewable"), true)
        )
      )
      .take(100);
    return result;
  },
});

export const getLatestTimestamp = internalQuery({
  handler: async (ctx) => {
    const record = await ctx.db
      .query("mails")
      .withIndex("by_timestamp")
      .order("desc")
      .first();
    return record?.timestamp;
  },
});

export const getMailByMessageId = internalQuery({
  args: { messageId: v.string() },
  handler: async (ctx, args) => {
    const record = await ctx.db
      .query("mails")
      .withIndex("by_messageId")
      .filter((q) => q.eq(q.field("messageId"), args.messageId))
      .first();
    return record;
  },
});
