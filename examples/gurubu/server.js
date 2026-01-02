// Gurubu Backend - Kiren Edition
// Migrated from Express.js to Kiren with WebSocket support

// Kiren Express compatibility
var express = require("../../lib/kiren-express.js");
var cors = express.cors;

// Initialize app
var app = express();

// In-memory storage
var groomings = {};
var users = [];
var wsConnections = {}; // Map ws.id -> { roomId, userId }

// Default metrics for voting (Fibonacci sequence for story points)
var DEFAULT_METRICS = [
  {
    id: 1,
    name: "storyPoint",
    order: 1,
    points: ["0", "1", "2", "3", "5", "8", "13", "21", "?", "☕"]
  }
];

// CORS options
var corsOptions = {
  origin: "*",
  credentials: true,
  methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"],
  allowedHeaders: ["Content-Type", "Authorization"]
};

// Middleware
app.use(express.json());
app.use(cors(corsOptions));

// Helper: Generate UUID
function uuid() {
  return crypto.randomUUID();
}

// Helper: Broadcast to room
function broadcastToRoom(roomId, message, excludeWsId) {
  var msgStr = JSON.stringify(message);
  for (var wsId in wsConnections) {
    var conn = wsConnections[wsId];
    if (conn.roomId === roomId) {
      if (excludeWsId && wsId == excludeWsId) continue;
      // Find ws connection and send
      // This will be handled by the WebSocket handler
    }
  }
  // Use Kiren broadcast to room
  Kiren.wsBroadcastRoom(roomId, msgStr);
}

// ============ HTTP ROUTES ============

// Health Check
app.get("/healthcheck", function(req, res) {
  res.json({
    status: "ok",
    runtime: "kiren",
    version: "0.1.0",
    binary: "754KB",
    features: ["http", "websocket"]
  });
});

// ============ ROOM ROUTES ============

// Create room
app.post("/room/create", function(req, res) {
  var nickname = req.body && req.body.nickname;
  var groomingType = req.body && req.body.groomingType;

  if (!nickname) {
    return res.status(400).json({ error: "Nickname is required" });
  }

  var roomId = uuid();
  var now = Date.now();

  groomings[roomId] = {
    roomID: roomId,
    groomingType: groomingType || 0,
    createdAt: now,
    expireTime: now + 12 * 60 * 60 * 1000,
    participants: [],
    votes: {},
    issues: [],
    currentIssue: null,
    showVotes: false
  };

  var user = {
    userID: users.length + 1,
    credentials: uuid(),
    nickname: nickname,
    roomID: roomId,
    connected: true,
    isAdmin: true
  };
  users.push(user);
  groomings[roomId].participants.push(user);

  res.json({
    roomId: roomId,
    credentials: user.credentials,
    userID: user.userID
  });
});

// Join room
app.post("/room/:roomId", function(req, res) {
  var roomId = req.params.roomId;
  var nickname = req.body && req.body.nickname;

  if (!nickname) {
    return res.status(400).json({ error: "Nickname is required" });
  }

  var room = groomings[roomId];
  if (!room) {
    return res.status(404).json({ error: "Room not found" });
  }

  if (Date.now() > room.expireTime) {
    delete groomings[roomId];
    return res.status(410).json({ error: "Room has expired" });
  }

  var user = {
    userID: users.length + 1,
    credentials: uuid(),
    nickname: nickname,
    roomID: roomId,
    connected: true,
    isAdmin: false
  };
  users.push(user);
  room.participants.push(user);

  res.json({
    roomId: roomId,
    credentials: user.credentials,
    userID: user.userID,
    groomingType: room.groomingType,
    participants: room.participants.length
  });
});

// Get room info
app.get("/room/:roomId", function(req, res) {
  var roomId = req.params.roomId;
  var room = groomings[roomId];

  if (!room) {
    return res.status(404).json({ error: "Room not found" });
  }

  res.json({
    roomId: room.roomID,
    groomingType: room.groomingType,
    participants: room.participants.map(function(p) {
      return { userID: p.userID, nickname: p.nickname, connected: p.connected };
    }),
    issues: room.issues,
    currentIssue: room.currentIssue,
    votes: room.showVotes ? room.votes : {},
    showVotes: room.showVotes,
    createdAt: room.createdAt
  });
});

// ============ VOTING ROUTES ============

// Submit vote
app.post("/room/:roomId/vote", function(req, res) {
  var roomId = req.params.roomId;
  var userId = req.body && req.body.userId;
  var vote = req.body && req.body.vote;

  var room = groomings[roomId];
  if (!room) {
    return res.status(404).json({ error: "Room not found" });
  }

  room.votes[userId] = vote;

  // Broadcast vote update
  broadcastToRoom(roomId, {
    type: "vote_update",
    userId: userId,
    hasVoted: true
  });

  res.json({ success: true });
});

// Reveal votes
app.post("/room/:roomId/reveal", function(req, res) {
  var roomId = req.params.roomId;
  var room = groomings[roomId];

  if (!room) {
    return res.status(404).json({ error: "Room not found" });
  }

  room.showVotes = true;

  // Broadcast reveal
  broadcastToRoom(roomId, {
    type: "votes_revealed",
    votes: room.votes
  });

  res.json({ success: true, votes: room.votes });
});

// Reset votes
app.post("/room/:roomId/reset", function(req, res) {
  var roomId = req.params.roomId;
  var room = groomings[roomId];

  if (!room) {
    return res.status(404).json({ error: "Room not found" });
  }

  room.votes = {};
  room.showVotes = false;

  // Broadcast reset
  broadcastToRoom(roomId, {
    type: "votes_reset"
  });

  res.json({ success: true });
});

// ============ STORY POINT ROUTES ============

app.post("/storypoint/estimate", function(req, res) {
  var description = req.body && req.body.description;

  if (!description) {
    return res.status(400).json({ error: "Description is required" });
  }

  res.json({
    estimation: 5,
    confidence: 0.8,
    reasoning: "Based on complexity analysis",
    suggestions: ["Consider breaking down if too large"]
  });
});

// ============ AI WORKFLOW ROUTES ============

app.post("/ai-workflow/estimate", function(req, res) {
  res.json({
    estimation: 8,
    confidence: 0.75,
    historical_comparison: "Similar to previous tasks",
    reasoning: "Multiple components involved"
  });
});

// ============ START SERVER ============

var PORT = 5001;

// Use Kiren.serve directly for combined HTTP + WebSocket
Kiren.serve({
  port: PORT,

  // HTTP handler
  fetch: function(req) {
    return handleHttpRequest(req);
  },

  // WebSocket handler
  websocket: {
    open: function(ws) {
      console.log("WS connected:", ws.id);
      wsConnections[ws.id] = { roomId: null, userId: null };

      Kiren.wsSend(ws, JSON.stringify({
        type: "connected",
        id: ws.id
      }));
    },

    message: function(ws, data) {
      var msg;
      try {
        msg = JSON.parse(data);
      } catch (e) {
        console.log("Invalid JSON:", data);
        return;
      }
      try {
        handleWsMessage(ws, msg);
      } catch (e) {
        console.log("Handler error for", msg.type + ":", e);
      }
    },

    close: function(ws) {
      console.log("WS disconnected:", ws.id);
      var conn = wsConnections[ws.id];
      if (conn && conn.roomId) {
        // Notify room that user left
        broadcastToRoom(conn.roomId, {
          type: "user_left",
          userId: conn.userId
        }, ws.id);
      }
      delete wsConnections[ws.id];
    }
  }
});

// HTTP request handler (using kiren-express internally)
function handleHttpRequest(rawReq) {
  var url = rawReq.url || "/";
  var method = rawReq.method || "GET";
  var path = url;
  var qIdx = url.indexOf("?");
  if (qIdx !== -1) path = url.substring(0, qIdx);

  // Build req/res for express-like handling
  var req = {
    method: method,
    url: url,
    path: path,
    params: {},
    query: {},
    body: null,
    headers: rawReq.headers || {}
  };

  // Parse body
  if (rawReq.body) {
    try { req.body = JSON.parse(rawReq.body); }
    catch (e) { req.body = rawReq.body; }
  }

  // Simple routing
  var resBody = "";
  var resStatus = 200;
  var resHeaders = { "Content-Type": "application/json" };

  var res = {
    status: function(code) { resStatus = code; return this; },
    json: function(data) { resBody = JSON.stringify(data); return this; },
    set: function(k, v) { resHeaders[k] = v; return this; }
  };

  // Add CORS headers
  resHeaders["Access-Control-Allow-Origin"] = "*";
  resHeaders["Access-Control-Allow-Methods"] = "GET, POST, PUT, DELETE, OPTIONS";
  resHeaders["Access-Control-Allow-Headers"] = "Content-Type, Authorization";

  if (method === "OPTIONS") {
    return new Response("", { status: 204, headers: resHeaders });
  }

  // Route matching
  if (method === "GET" && path === "/healthcheck") {
    res.json({ status: "ok", runtime: "kiren", version: "0.1.0", binary: "754KB", features: ["http", "websocket"] });
  }
  else if (method === "POST" && path === "/room/create") {
    var nickname = req.body && (req.body.nickname || req.body.nickName);
    var groomingType = req.body && (req.body.groomingType || 0);
    if (!nickname) {
      res.status(400).json({ error: "Nickname is required" });
    } else {
      var roomId = uuid();
      var now = Date.now();
      groomings[roomId] = {
        roomID: roomId, groomingType: groomingType || 0, createdAt: now,
        expireTime: now + 12 * 60 * 60 * 1000, participants: [], votes: {},
        issues: [], currentIssue: null, showVotes: false
      };
      var user = { userID: users.length + 1, credentials: uuid(), nickname: nickname, roomID: roomId, connected: true, isAdmin: true, sockets: [], votes: {} };
      users.push(user);
      groomings[roomId].participants.push(user);
      res.status(201).json({ roomID: roomId, credentials: user.credentials, userID: user.userID, isAdmin: true });
    }
  }
  else if (method === "GET" && path.match(/^\/room\/[a-f0-9-]+$/)) {
    var roomId = path.split("/")[2];
    var room = groomings[roomId];
    if (!room) {
      res.status(404).json({ error: "Room not found" });
    } else {
      res.json({
        roomId: room.roomID, groomingType: room.groomingType,
        participants: room.participants.map(function(p) { return { userID: p.userID, nickname: p.nickname, connected: p.connected }; }),
        issues: room.issues, currentIssue: room.currentIssue,
        votes: room.showVotes ? room.votes : {}, showVotes: room.showVotes, createdAt: room.createdAt
      });
    }
  }
  else if (method === "POST" && path.match(/^\/room\/[a-f0-9-]+$/)) {
    var roomId = path.split("/")[2];
    var nickname = req.body && (req.body.nickname || req.body.nickName);
    if (!nickname) {
      res.status(400).json({ error: "Nickname is required" });
    } else {
      var room = groomings[roomId];
      if (!room) {
        res.status(404).json({ error: "Room not found" });
      } else {
        var user = { userID: users.length + 1, credentials: uuid(), nickname: nickname, roomID: roomId, connected: true, isAdmin: false, sockets: [], votes: {} };
        users.push(user);
        room.participants.push(user);
        res.json({ roomID: roomId, credentials: user.credentials, userID: user.userID, groomingType: room.groomingType, participants: room.participants.length, isAdmin: false });
      }
    }
  }
  else if (method === "POST" && path.match(/^\/room\/[a-f0-9-]+\/vote$/)) {
    var roomId = path.split("/")[2];
    var userId = req.body && req.body.userId;
    var vote = req.body && req.body.vote;
    var room = groomings[roomId];
    if (!room) {
      res.status(404).json({ error: "Room not found" });
    } else {
      room.votes[userId] = vote;
      broadcastToRoom(roomId, { type: "vote_update", userId: userId, hasVoted: true });
      res.json({ success: true });
    }
  }
  else if (method === "POST" && path.match(/^\/room\/[a-f0-9-]+\/reveal$/)) {
    var roomId = path.split("/")[2];
    var room = groomings[roomId];
    if (!room) {
      res.status(404).json({ error: "Room not found" });
    } else {
      room.showVotes = true;
      broadcastToRoom(roomId, { type: "votes_revealed", votes: room.votes });
      res.json({ success: true, votes: room.votes });
    }
  }
  else if (method === "POST" && path.match(/^\/room\/[a-f0-9-]+\/reset$/)) {
    var roomId = path.split("/")[2];
    var room = groomings[roomId];
    if (!room) {
      res.status(404).json({ error: "Room not found" });
    } else {
      room.votes = {};
      room.showVotes = false;
      broadcastToRoom(roomId, { type: "votes_reset" });
      res.json({ success: true });
    }
  }
  else if (method === "POST" && path === "/storypoint/estimate") {
    var description = req.body && req.body.description;
    if (!description) {
      res.status(400).json({ error: "Description is required" });
    } else {
      res.json({ estimation: 5, confidence: 0.8, reasoning: "Based on complexity analysis", suggestions: ["Consider breaking down if too large"] });
    }
  }
  else if (method === "POST" && path === "/ai-workflow/estimate") {
    res.json({ estimation: 8, confidence: 0.75, historical_comparison: "Similar to previous tasks", reasoning: "Multiple components involved" });
  }
  else {
    res.status(404).json({ error: "Not Found" });
  }

  return new Response(resBody, { status: resStatus, headers: resHeaders });
}

// Helper: Build grooming info response
function buildGroomingInfo(room) {
  return {
    roomID: room.roomID,
    participants: room.participants,
    votes: room.votes,
    isResultShown: room.showVotes,
    mode: room.groomingType,
    issues: room.issues || [],
    currentIssue: room.currentIssue,
    metrics: room.metrics || DEFAULT_METRICS
  };
}

// WebSocket message handler
function handleWsMessage(ws, msg) {
  var conn = wsConnections[ws.id];
  var msgType = msg.type;
  var msgData = msg.data;

  console.log("WS message:", msgType, JSON.stringify(msgData));

  switch (msgType) {
    case "joinRoom":
      // Frontend sends: { nickname, roomID, lobby }
      var joinData = msgData;
      var roomId = joinData.roomID;
      var lobby = joinData.lobby;

      conn.roomId = roomId;
      conn.credentials = lobby ? lobby.credentials : null;
      conn.userId = lobby ? lobby.userID : null;
      conn.nickname = joinData.nickname;

      console.log("joinRoom: roomId=" + roomId + ", userId=" + conn.userId);
      console.log("joinRoom: groomings keys=" + Object.keys(groomings).join(", "));

      // Join the WebSocket room
      Kiren.wsJoinRoom(ws, roomId);

      var room = groomings[roomId];
      console.log("joinRoom: room found=" + (room ? "yes" : "no"));
      if (room) {
        console.log("joinRoom: participants=" + JSON.stringify(room.participants));
        // Update participant connection status and add socket
        for (var i = 0; i < room.participants.length; i++) {
          if (room.participants[i].userID === conn.userId) {
            room.participants[i].connected = true;
            if (!room.participants[i].sockets) {
              room.participants[i].sockets = [];
            }
            room.participants[i].sockets.push(ws.id);
            if (!room.participants[i].votes) {
              room.participants[i].votes = {};
            }
            break;
          }
        }
        // Send current room state to joiner
        var groomingInfo = buildGroomingInfo(room);
        console.log("joinRoom: sending voteSent=" + JSON.stringify(groomingInfo));
        Kiren.wsSend(ws, JSON.stringify({
          type: "voteSent",
          data: groomingInfo
        }));
        // Broadcast updated state to room
        broadcastToRoom(roomId, {
          type: "voteSent",
          data: buildGroomingInfo(room)
        });
      }
      break;

    case "userVote":
      // Frontend sends: [voteObj, roomID, credentials]
      var voteObj = msgData[0];
      var roomId = msgData[1];
      var credentials = msgData[2];

      var room = groomings[roomId];
      if (room) {
        // Find user by credentials
        var user = null;
        for (var i = 0; i < room.participants.length; i++) {
          if (room.participants[i].credentials === credentials) {
            user = room.participants[i];
            break;
          }
        }
        if (user) {
          user.votes = voteObj;
          room.votes[user.userID] = voteObj;
        }
        // Broadcast updated state
        broadcastToRoom(roomId, {
          type: "voteSent",
          data: buildGroomingInfo(room)
        });
      }
      break;

    case "showResults":
      // Frontend sends: [roomId, credentials]
      var roomId = msgData[0];
      var credentials = msgData[1];

      var room = groomings[roomId];
      if (room) {
        room.showVotes = true;
        broadcastToRoom(roomId, {
          type: "showResults",
          data: buildGroomingInfo(room)
        });
      }
      break;

    case "resetVotes":
      // Frontend sends: [roomId, credentials]
      var roomId = msgData[0];
      var credentials = msgData[1];

      var room = groomings[roomId];
      if (room) {
        room.votes = {};
        room.showVotes = false;
        // Clear participant votes
        for (var i = 0; i < room.participants.length; i++) {
          room.participants[i].votes = {};
        }
        broadcastToRoom(roomId, {
          type: "resetVotes",
          data: buildGroomingInfo(room)
        });
      }
      break;

    case "ping":
      Kiren.wsSend(ws, JSON.stringify({ type: "pong" }));
      break;

    case "updateAvatar":
      // Frontend sends: [roomID, avatarConfig, credentials]
      var roomId = msgData[0];
      var avatarConfig = msgData[1];
      var credentials = msgData[2];

      var room = groomings[roomId];
      if (room) {
        // Find user by credentials and update avatar
        for (var i = 0; i < room.participants.length; i++) {
          if (room.participants[i].credentials === credentials) {
            room.participants[i].avatar = avatarConfig;
            break;
          }
        }
        // Broadcast avatar update
        broadcastToRoom(roomId, {
          type: "avatarUpdated",
          data: {
            credentials: credentials,
            avatar: avatarConfig,
            participants: room.participants
          }
        });
      }
      break;
  }
}

console.log("");
console.log("========================================");
console.log("   Gurubu Backend - Kiren Edition");
console.log("========================================");
console.log("");
console.log("Server: http://localhost:" + PORT);
console.log("WebSocket: ws://localhost:" + PORT);
console.log("Runtime: Kiren (754KB binary)");
console.log("");
console.log("HTTP Endpoints:");
console.log("  GET  /healthcheck");
console.log("  POST /room/create");
console.log("  POST /room/:roomId");
console.log("  GET  /room/:roomId");
console.log("  POST /storypoint/estimate");
console.log("  POST /ai-workflow/estimate");
console.log("");
console.log("WebSocket Events:");
console.log("  join_room, vote, reveal_votes, reset_votes");
console.log("");
