import { GenericActionCtx } from "convex/server";
import { Api as BotApi } from "grammy";
import { Update } from "grammy/types";
import { telegramSecrets } from "../secrets";
import { internal } from "./_generated/api";
import {
  httpAction,
  internalAction
} from "./_generated/server";

const telegramBot = new BotApi(telegramSecrets.botToken);

export const receiveMessage = httpAction(async (ctx, req) => {
  const telegramMessage: Update = await req.json();

  if (
    telegramSecrets.userId &&
    telegramMessage.message?.from.id !== telegramSecrets.userId
  ) {
    console.warn(
      "sender->",
      telegramMessage.message,
      "and my user->",
      telegramSecrets.userId
    );
  } else {
    if (telegramMessage.message?.entities?.[0]?.type === "bot_command") {
      await handleBotCommands(ctx, telegramMessage.message.text ?? "");
    }
  }

  console.log("received request", telegramMessage);
  return new Response(null, { status: 200 });
});

export const sendMailsInMessages = internalAction(async (ctx) => {
  const mails = await ctx.runQuery(internal.gmail.queries.getMails);

  if (mails.length === 0) {
    await telegramBot.sendMessage(
      telegramSecrets.userId,
      "No new mails found!"
    );
    return;
  }

  // query all the records in the mails table
  await Promise.all(
    mails.map(async (mail) => {
      await telegramBot.sendMessage(telegramSecrets.userId, mail.hash);
      await ctx.runMutation(internal.gmail.mutations.markMessageAsSent, {
        id: mail._id,
      });
    })
  );

  // 2. send the link to the webpage in the message
  // 3. simultaneously can also send a link to telegram mini app
});

const handleBotCommands = async (
  ctx: GenericActionCtx<any>,
  command: string
) => {
  if (command === "/fetch") {
    await telegramBot.sendMessage(
      telegramSecrets.userId,
      "Fetching new mails in progress..."
    );
    await ctx.scheduler.runAfter(0, internal.gmail.actions.fetchGmails);
  } else if (command === "/quickfetch") {
    await ctx.scheduler.runAfter(0, internal.telegram.sendMailsInMessages);
  }
};
