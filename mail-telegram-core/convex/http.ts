import { httpRouter } from "convex/server";
import { getMessage, receiveMessage } from "./telegram";

const http = httpRouter();

http.route({
  path: "/get-message",
  method: "GET",
  handler: getMessage,
});

http.route({
  path: '/telegram',
  method: 'POST',
  handler: receiveMessage
})

export default http;