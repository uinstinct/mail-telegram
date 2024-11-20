import fs from 'fs/promises';
import path from 'path';
import { authenticate } from '@google-cloud/local-auth';
import { google, gmail_v1 } from 'googleapis';
import { OAuth2Client } from 'google-auth-library';
import { GaxiosPromise } from 'googleapis/build/src/apis/abusiveexperiencereport';

// Type Definitions
interface MessageDetails {
  subject: string;
  from: string;
  date: string;
  body: string;
}

interface Credentials {
  installed?: {
    client_id: string;
    client_secret: string;
    redirect_uris: string[];
  };
  web?: {
    client_id: string;
    client_secret: string;
    redirect_uris: string[];
  };
}

interface GmailContext {
  auth: OAuth2Client;
  service: gmail_v1.Gmail;
}

// Configuration Constants
const SCOPES = ['https://www.googleapis.com/auth/gmail.readonly'];
const TOKEN_PATH = path.join(process.cwd(), 'token.json');
const CREDENTIALS_PATH = path.join(process.cwd(), 'credentials.json');

/**
 * Load saved credentials from file
 */
async function loadSavedCredentials(): Promise<OAuth2Client | null> {
  try {
    const content = await fs.readFile(TOKEN_PATH, 'utf8');
    const credentials = JSON.parse(content);
    return google.auth.fromJSON(credentials) as OAuth2Client;
  } catch (err) {
    return null;
  }
}

/**
 * Save credentials to a file
 */
async function saveCredentials(client: OAuth2Client): Promise<void> {
  try {
    const content = await fs.readFile(CREDENTIALS_PATH, 'utf8');
    const keys: Credentials = JSON.parse(content);
    const key = keys.installed || keys.web;

    if(!key) {
      throw new Error('No key found in credentials file');
    }

    const payload = {
      type: 'authorized_user',
      client_id: key.client_id,
      client_secret: key.client_secret,
      refresh_token: client.credentials.refresh_token,
    };

    await fs.writeFile(TOKEN_PATH, JSON.stringify(payload));
  } catch (err) {
    console.error('Error saving credentials:', err);
  }
}

/**
 * Authenticate and create Gmail context
 */
async function createGmailContext(): Promise<GmailContext> {
  // Try to load existing credentials
  let client = await loadSavedCredentials();

  if (!client) {
    // If no saved credentials, authenticate
    client = await authenticate({
      scopes: SCOPES,
      keyfilePath: CREDENTIALS_PATH,
    }) as OAuth2Client;

    // Save the credentials if possible
    if (client.credentials) {
      await saveCredentials(client);
    }
  }

  // Create Gmail service
  const service = google.gmail({ version: 'v1', auth: client });

  return { auth: client, service };
}

/**
 * List messages from inbox
 */
async function listMessages(context: GmailContext): Promise<gmail_v1.Schema$Message[]> {
  const messages: gmail_v1.Schema$Message[] = [];
  let pageToken: string | undefined = undefined;

  do {
    const response: any = await context.service.users.messages.list({
    userId: 'me',
      labelIds: ['INBOX'],
      pageToken: pageToken,
      maxResults: 100
    });

    if (response.data.messages) {
      messages.push(...response.data.messages);
    }

    pageToken = response.data.nextPageToken || undefined;
  } while (pageToken);

  return messages;
}

/**
 * Extract headers from message
 */
function extractHeaders(headers: gmail_v1.Schema$MessagePartHeader[] = []): 
  Record<string, string> {
  return headers.reduce((acc, header) => {
    if (header.name && header.value) {
      acc[header.name.toLowerCase()] = header.value;
    }
    return acc;
  }, {} as Record<string, string>);
}

/**
 * Extract message body
 */
function extractMessageBody(payload?: gmail_v1.Schema$MessagePart): string {
  if (!payload) return 'No content';

  // Check for plain text part in multipart messages
  if (payload.parts) {
    const plainTextPart = payload.parts.find(part => part.mimeType === 'text/plain');
    if (plainTextPart?.body?.data) {
      return Buffer.from(plainTextPart.body.data, 'base64').toString('utf-8');
    }
  }

  // Fallback to main body
  if (payload.body?.data) {
    return Buffer.from(payload.body.data, 'base64').toString('utf-8');
  }

  return 'No content';
}

/**
 * Get details of a specific message
 */
async function getMessageDetails(
  context: GmailContext, 
  messageId: string
): Promise<MessageDetails | null> {
  try {
    const response = await context.service.users.messages.get({
      userId: 'me',
      id: messageId,
      format: 'full'
    });

    const message = response.data;
    const headers = extractHeaders(message.payload?.headers);

    return {
      subject: headers['subject'] || 'No Subject',
      from: headers['from'] || 'Unknown Sender',
      date: headers['date'] || 'No Date',
      body: extractMessageBody(message.payload)
    };
  } catch (error) {
    console.error('Error fetching message details:', error);
    return null;
  }
}

/**
 * Fetch inbox messages
 */
async function fetchInboxMessages(
  context: GmailContext, 
  limit: number = 10
): Promise<MessageDetails[]> {
  // Fetch messages
  const messages = await listMessages(context);
  console.log(`Found ${messages.length} messages in inbox`);

  // Fetch details for messages
  const messageDetails = await Promise.all(
    messages.slice(0, limit).map(msg => getMessageDetails(context, msg.id!))
  );

  // Filter out null results
  return messageDetails.filter((detail): detail is MessageDetails => detail !== null);
}

/**
 * Display messages
 */
function displayMessages(messages: MessageDetails[]): void {
  messages.forEach((details, index) => {
    console.log('\n' + '='.repeat(50));
    console.log(`Email ${index + 1}:`);
    console.log(`From: ${details.from}`);
    console.log(`Subject: ${details.subject}`);
    console.log(`Date: ${details.date}`);
    console.log('-'.repeat(50));
    console.log(`Body: ${details.body.substring(0, 200)}...`);
  });
}

/**
 * Main function to orchestrate the email fetching process
 */
async function main(): Promise<void> {
  try {
    // Create Gmail context
    const gmailContext = await createGmailContext();

    // Fetch and display messages
    const messages = await fetchInboxMessages(gmailContext);
    displayMessages(messages);
  } catch (error) {
    console.error('Error in Gmail API process:', error);
  }
}

// Run the script
main();
