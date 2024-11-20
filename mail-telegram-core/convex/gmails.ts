"use node";

import { action, httpAction, internalAction } from "./_generated/server";
import { authenticate } from "@google-cloud/local-auth";
import fs from "fs";
import path from "path";
import { google, gmail_v1 } from "googleapis";
import { OAuth2Client } from "google-auth-library";
import { internal } from "./_generated/api";
import gmailTokenCreds from './gmail_token'

const getGmailService = async () => {
    const gmailClient = google.auth.fromJSON(gmailTokenCreds) as OAuth2Client;
    const gmailService = google.gmail({ version: "v1", auth: gmailClient });
    return gmailService;
}

export const fetchGmails = internalAction({
  handler: async () => {
    const gmailService = await getGmailService();
    const messages = await gmailService.users.messages.list({
      userId: "me",
      labelIds: ["CATEGORY_PERSONAL"],
      maxResults: 100,
      includeSpamTrash: false,
      q: 'is:unread',
    });
    if(!messages.data.messages) {
      return []
    }
    const htmls = await Promise.all(messages.data.messages.map(async (message) => {
      if(!message.id) {
        return null
      }
      const currentMessage = await gmailService.users.messages.get({
        userId: "me",
        id: message.id,
        format: "full",
      });
      return Buffer.from(currentMessage.data.payload?.parts?.[1].body?.data ?? '', "base64").toString("utf-8")
    }))
    return htmls
  },
});
