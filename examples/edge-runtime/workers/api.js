export default {
  fetch(request) {
    console.log("Request:", request.method, request.url);

    if (request.url.includes("/api/hello")) {
      return Response.json({ message: "Hello from Kiren Edge!" });
    }

    if (request.url.includes("/api/time")) {
      return Response.json({ time: new Date().toISOString() });
    }

    return Response.json({ error: "Not found" }, { status: 404 });
  }
};
