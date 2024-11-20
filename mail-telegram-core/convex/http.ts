import { httpRouter } from "convex/server";
import { getMessage } from "./telegram";

const http = httpRouter();

http.route({
  path: "/get-message",
  method: "GET",
  handler: getMessage,
});

export default http;