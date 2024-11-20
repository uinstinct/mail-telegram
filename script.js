const fs = require('fs').promises;
const path = require('path');
const process = require('process');
const {authenticate} = require('@google-cloud/local-auth');
const {google} = require('googleapis');

// If modifying these scopes, delete token.json.
const SCOPES = ['https://www.googleapis.com/auth/gmail.readonly'];
// The file token.json stores the user's access and refresh tokens
const TOKEN_PATH = path.join(process.cwd(), 'token.json');
const CREDENTIALS_PATH = path.join(process.cwd(), 'credentials.json');

/**
 * Reads previously authorized credentials from the save file.
 */
async function loadSavedCredentialsIfExist() {
  try {
    const content = await fs.readFile(TOKEN_PATH);
    const credentials = JSON.parse(content);
    return google.auth.fromJSON(credentials);
  } catch (err) {
    return null;
  }
}

/**
 * Serializes credentials to a file compatible with GoogleAuth.fromJSON.
 */
async function saveCredentials(client) {
  const content = await fs.readFile(CREDENTIALS_PATH);
  const keys = JSON.parse(content);
  const key = keys.installed || keys.web;
  const payload = {
    type: 'authorized_user',
    client_id: key.client_id,
    client_secret: key.client_secret,
    refresh_token: client.credentials.refresh_token,
  };
  await fs.writeFile(TOKEN_PATH, JSON.stringify(payload));
}

/**
 * Load or request authorization to call APIs.
 */
async function authorize() {
  let client = await loadSavedCredentialsIfExist();
  if (client) {
    return client;
  }
  client = await authenticate({
    scopes: SCOPES,
    keyfilePath: CREDENTIALS_PATH,
  });
  if (client.credentials) {
    await saveCredentials(client);
  }
  return client;
}

/**
 * Lists all messages in the user's inbox
 */
async function listMessages(auth) {
  const gmail = google.gmail({version: 'v1', auth});
  const messages = [];
  
  try {
    let pageToken = null;
    do {
      const response = await gmail.users.messages.list({
        userId: 'me',
        labelIds: ['INBOX'],
        pageToken: pageToken,
        maxResults: 100
      });
      
      if (response.data.messages) {
        messages.push(...response.data.messages);
      }
      
      pageToken = response.data.nextPageToken;
    } while (pageToken);
    
    return messages;
  } catch (error) {
    console.error('Error fetching messages:', error.message);
    return [];
  }
}

/**
 * Gets the details of a specific message
 */
async function getMessageDetails(auth, messageId) {
  const gmail = google.gmail({version: 'v1', auth});
  
  try {
    const response = await gmail.users.messages.get({
      userId: 'me',
      id: messageId,
      format: 'full'
    });
    
    const message = response.data;
    const headers = message.payload.headers;
    
    const subject = headers.find(header => header.name.toLowerCase() === 'subject')?.value || 'No Subject';
    const from = headers.find(header => header.name.toLowerCase() === 'from')?.value || 'Unknown Sender';
    const date = headers.find(header => header.name.toLowerCase() === 'date')?.value || 'No Date';
    
    // Get message body
    let body = '';
    if (message.payload.parts) {
      const part = message.payload.parts.find(part => part.mimeType === 'text/plain');
      if (part && part.body.data) {
        body = Buffer.from(part.body.data, 'base64').toString('utf-8');
      }
    } else if (message.payload.body.data) {
      body = Buffer.from(message.payload.body.data, 'base64').toString('utf-8');
    }
    
    return {
      subject,
      from,
      date,
      body: body || 'No content'
    };
  } catch (error) {
    console.error('Error fetching message details:', error.message);
    return null;
  }
}

/**
 * Main function to run the script
 */
async function main() {
  try {
    // Authenticate
    const auth = await authorize();
    console.log('Authentication successful!');
    
    // Get all messages
    const messages = await listMessages(auth);
    console.log(`Found ${messages.length} messages in inbox`);
    
    // Get details for first 10 messages
    const messageDetails = await Promise.all(
      messages.slice(0, 10).map(msg => getMessageDetails(auth, msg.id))
    );
    
    // Display results
    messageDetails.forEach((details, index) => {
      if (details) {
        console.log('\n' + '='.repeat(50));
        console.log(`Email ${index + 1}:`);
        console.log(`From: ${details.from}`);
        console.log(`Subject: ${details.subject}`);
        console.log(`Date: ${details.date}`);
        console.log('-'.repeat(50));
        console.log(`Body: ${details.body.substring(0, 200)}...`);
      }
    });
    
  } catch (error) {
    console.error('Error:', error.message);
  }
}

// Run the script
main();