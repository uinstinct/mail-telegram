import { internalQuery } from "../_generated/server";


export const getMails = internalQuery({
    handler: async (ctx) => {
        const result = await ctx.db.query("mails").take(100);
        return result;
    },
});