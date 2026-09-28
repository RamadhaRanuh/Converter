"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = __importDefault(require("express"));
const cors_1 = __importDefault(require("cors"));
const path_1 = __importDefault(require("path"));
const fs_1 = __importDefault(require("fs"));
const imageRoutes_1 = require("./routers/imageRoutes");
// Create Express app
const app = (0, express_1.default)();
const PORT = process.env.PORT || 3000;
// MIDDLEWARE SECTION
app.use((0, cors_1.default)({
    origin: ['http://localhost:5173', 'http://127.0.0.1:5173'], // Vite's default port
    methods: ['GET', 'POST', 'PUT', 'DELETE'],
    allowedHeaders: ['Content-Type', 'Authorization']
})); // Enable Cross-Origin Resource Sharing
/*
Example: if your backend is at http://localhost:3000
and your frontend is at http://localhost:4200,
the frontend can make requests to the backend
*/
app.use(express_1.default.json()); // Parse incoming JSON payloads from requests
/*
Example: if your frontend sends a POST request with a JSON payload,
you can access the data in the request body using req.body
*/
app.use(express_1.default.urlencoded({ extended: true })); // Parse incoming URL-encoded payloads from requests
// make sure uploads directory exists
const uploadsDir = path_1.default.join(__dirname, '..', 'uploads');
if (!fs_1.default.existsSync(uploadsDir)) // If the uploads directory does not exist, create it
 {
    fs_1.default.mkdirSync(uploadsDir, { recursive: true });
}
// Serve static files from the uploads directory
app.use('/uploads', express_1.default.static(uploadsDir));
// ROUTES SECTION
app.use('/api/images', imageRoutes_1.ImageRouter);
// Basic route
app.get('/', (req, res) => {
    res.send('Image Converter API is running');
});
// Start Server
app.listen(PORT, () => {
    console.log(`Server running on port ${PORT}`);
});
exports.default = app; // Export the app for testing purposes
