import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
  mails: defineTable({
    content: v.string(),
    uid: v.string(),
  }),
});