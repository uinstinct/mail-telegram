import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
  mails: defineTable({
    content: v.string(),
    hash: v.string(),
    timestamp: v.string(),
    subject: v.string(),
    from: v.string(),
    messageId: v.string(),
    sentOnTelegram: v.boolean(),
    isViewable: v.boolean(),
  })
    .index("by_messageId", ["messageId"])
    .index("by_timestamp", ["timestamp"]),
});
