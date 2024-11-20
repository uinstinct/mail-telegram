import { Bot } from "grammy";
import { internal } from "./_generated/api";
import { httpAction, internalAction } from "./_generated/server";
import { telegramSecrets } from "../secrets";
import { Update } from "grammy/types";

export const telegramBot = new Bot(telegramSecrets.botToken);

export const getMessage = httpAction(async (ctx) => {
  const htmls = await ctx.runAction(internal.gmails.fetchGmails);
  return Response.json(htmls);
});

export const receiveMessage = httpAction(async (ctx, req) => {
  const telegramMessage: Update = await req.json();

  if (
    telegramSecrets.userId &&
    telegramMessage.message?.from.id !== telegramSecrets.userId
  ) {
      console.warn('sender->', telegramMessage.message, 'and my user->', telegramSecrets.userId)
  } else {
      await ctx.scheduler.runAfter(0, internal.telegram.sendMessage)
  }
  return new Response(null, { status: 200 });
});

export const sendMessage = internalAction(async (ctx) => {
    const htmls = await ctx.runAction(internal.gmails.fetchGmails);
    // 1. generate a snowflake or uuid
    // 2. send the link to the webpage in the message
    // 3. simultaneously can also send a link to telegram mini app
    await telegramBot.api.sendMessage(telegramSecrets.userId, JSON.stringify(htmls[0]?.slice(0, 200)))
})