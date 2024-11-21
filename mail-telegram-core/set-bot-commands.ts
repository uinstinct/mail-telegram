import { Api as BotApi } from "grammy";
import { telegramSecrets } from "./secrets";

/**not functional currently */
async function setBotCommands() {
  console.log("setting the bot commands");
  const telegramBot = new BotApi(telegramSecrets.botToken);
  await telegramBot.setMyCommands([
    { command: "fetch", description: "fetch new mails" },
    { command: "quickfetch", description: "quickly fetch from database" },
  ]);
}

setBotCommands();
