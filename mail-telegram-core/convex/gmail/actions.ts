"use node";

import { v } from "convex/values";
import { OAuth2Client } from "google-auth-library";
import { gmail_v1, google } from "googleapis";
import { gmailTokenCreds } from "../../secrets";
import { internal } from "../_generated/api";
import {
  internalAction
} from "../_generated/server";

const getGmailService = async () => {
  const gmailClient = google.auth.fromJSON(gmailTokenCreds) as OAuth2Client;
  const gmailService = google.gmail({ version: "v1", auth: gmailClient });
  return gmailService;
};

const extractEssentialFieldsFromMessage = async (
  gmailService: gmail_v1.Gmail,
  message: gmail_v1.Schema$Message
) => {
  const uniqueMessageId = message.id!;
  const currentMessage = await gmailService.users.messages.get({
    userId: "me",
    id: message.id!,
    format: "full",
  });
  const content = Buffer.from(
    currentMessage.data.payload?.parts?.[1].body?.data ||
      currentMessage.data.payload?.parts?.[0].body?.data ||
      "",
    "base64"
  ).toString("utf-8");
  const subject =
    currentMessage.data.payload?.headers?.find(
      (header) => header.name === "Subject"
    )?.value || "Failed to get subject";
  const sender =
    currentMessage.data.payload?.headers?.find(
      (header) =>
        header.name === "X-SimpleLogin-Original-From" || header.name === "From"
    )?.value || "Failed to get sender";
  const receivedTimestamp =
    currentMessage.data.internalDate || "Failed to get timestamp";

  return {
    uniqueMessageId,
    content,
    subject,
    sender,
    receivedTimestamp,
  };
};

export const fetchGmails = internalAction({
  handler: async (ctx, ) => {
    const gmailService = await getGmailService();
    const latestTimestamp = await ctx.runQuery(
      internal.gmail.queries.getLatestTimestamp
    );
    console.log("fetching with latestTimestamp", latestTimestamp);

    const messages = await gmailService.users.messages.list({
      userId: "me",
      labelIds: ["CATEGORY_PERSONAL"],
      maxResults: 100,
      includeSpamTrash: false,
      q: latestTimestamp
        ? `is:unread after:${Math.floor(parseInt(latestTimestamp, 10) / 1000)}`
        : "is:unread",
    });
    if (!messages.data.messages) {
      return [];
    }
    console.log('fetched ',messages.data.messages.length, 'messages');
    
    await Promise.all(
      messages.data.messages.map(async (message) => {
        if (!message.id) return;

        const foundDuplicateMail = await ctx.runQuery(
          internal.gmail.queries.getMailByMessageId,
          { messageId: message.id }
        );
        if (foundDuplicateMail) {
          console.warn("found duplicate mail", foundDuplicateMail?.messageId);
          return;
        }

        const { uniqueMessageId, content, subject, sender, receivedTimestamp } =
          await extractEssentialFieldsFromMessage(gmailService, message);

        await ctx.runMutation(internal.gmail.mutations.storeMailsInDB, {
          content, // need to check the content is not greater than 1MB
          from: sender,
          messageId: uniqueMessageId,
          subject,
          timestamp: receivedTimestamp,
        });
      })
    );

    await ctx.scheduler.runAfter(0, internal.telegram.sendMailsInMessages)
  },
});
