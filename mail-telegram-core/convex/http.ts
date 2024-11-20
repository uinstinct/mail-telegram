import { httpRouter } from "convex/server";
import {  receiveMessage } from "./telegram";

const http = httpRouter();
http.route({
  path: '/telegram',
  method: 'POST',
  handler: receiveMessage
})

export default http;