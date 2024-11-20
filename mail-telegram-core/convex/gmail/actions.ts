"use node";

import {
  action,
  httpAction,
  internalAction,
  internalMutation,
} from "../_generated/server";
import { authenticate } from "@google-cloud/local-auth";
import fs from "fs";
import path from "path";
import { google, gmail_v1 } from "googleapis";
import { OAuth2Client } from "google-auth-library";
import { internal } from "../_generated/api";
import { gmailTokenCreds } from "../../secrets";
import { v } from "convex/values";

const getGmailService = async () => {
  const gmailClient = google.auth.fromJSON(gmailTokenCreds) as OAuth2Client;
  const gmailService = google.gmail({ version: "v1", auth: gmailClient });
  return gmailService;
};


export const fetchGmails = internalAction({
  handler: async (ctx) => {
    const gmailService = await getGmailService();
    const messages = await gmailService.users.messages.list({
      userId: "me",
      labelIds: ["CATEGORY_PERSONAL"],
      maxResults: 100,
      includeSpamTrash: false,
      q: "is:unread",
    });
    if (!messages.data.messages) {
      return [];
    }
    await Promise.all(
      messages.data.messages.map(async (message) => {
        if (!message.id) {
          return null;
        }
        const currentMessage = await gmailService.users.messages.get({
          userId: "me",
          id: message.id,
          format: "full",
        });
        const content = Buffer.from(
          currentMessage.data.payload?.parts?.[1].body?.data ||
            currentMessage.data.payload?.parts?.[0].body?.data ||
            "",
          "base64"
        ).toString("utf-8");
        
        await ctx.runMutation(internal.gmail.mutations.storeMailsInDB, { content});
      })
    );
  },
});