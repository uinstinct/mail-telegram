import { internal } from "./_generated/api"
import { httpAction } from "./_generated/server"

export const getMessage = httpAction(async (ctx)=>{
    const htmls = await ctx.runAction(internal.gmails.fetchGmails)
    return new Response(JSON.stringify(htmls), {
      status: 200
    })
  })
  