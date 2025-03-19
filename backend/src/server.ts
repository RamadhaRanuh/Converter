import express from 'express';
import cors from 'cors';
import path from 'path';
import fs from 'fs';
import { ImageRouter } from './routers/imageRoutes';


// Create Express app
const app = express();
const PORT = process.env.PORT || 3000;

// MIDDLEWARE SECTION
app.use(cors({
    origin: ['http://localhost:5173', 'http://127.0.0.1:5173'], // Vite's default port
    methods: ['GET', 'POST', 'PUT', 'DELETE'],
    allowedHeaders: ['Content-Type', 'Authorization']
  })); // Enable Cross-Origin Resource Sharing

/*
Example: if your backend is at http://localhost:3000 
and your frontend is at http://localhost:4200, 
the frontend can make requests to the backend
*/

app.use(express.json()); // Parse incoming JSON payloads from requests

/*
Example: if your frontend sends a POST request with a JSON payload,
you can access the data in the request body using req.body
*/

app.use(express.urlencoded({ extended: true })); // Parse incoming URL-encoded payloads from requests

// make sure uploads directory exists
const uploadsDir = path.join(__dirname, '..', 'uploads');

if (!fs.existsSync(uploadsDir)) // If the uploads directory does not exist, create it
{
    fs.mkdirSync(uploadsDir, { recursive: true });
}

// Serve static files from the uploads directory
app.use('/uploads', express.static(uploadsDir));

// ROUTES SECTION
app.use('/api/images', ImageRouter); 

// Basic route
app.get('/', (req, res) => {
    res.send('Image Converter API is running');
})

// Start Server
app.listen(PORT, () => {
    console.log(`Server running on port ${PORT}`);
})

export default app; // Export the app for testing purposes



