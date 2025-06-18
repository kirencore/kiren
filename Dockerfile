# Kiren Production Container
# Usage: docker build -t kiren . && docker run -p 3000:3000 kiren

FROM scratch

# Copy the Kiren binary
COPY target/release/kiren /kiren

# Copy demo application
COPY examples/production-demo.js /app.js

# Expose port
EXPOSE 3000

# Set environment variables
ENV NODE_ENV=production
ENV PORT=3000

# Run the application
CMD ["/kiren", "/app.js"]