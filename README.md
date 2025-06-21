<h1 align="center">Converter</h1>

-----

### Built with the tools and technologies:

<p align="center">
  <img src="https://img.shields.io/badge/-TypeScript-3178C6?logo=typescript&logoColor=white" alt="TypeScript">
  <img src="https://img.shields.io/badge/-JavaScript-F7DF1E?logo=javascript&logoColor=black" alt="JavaScript">
  <img src="https://img.shields.io/badge/-React-61DAFB?logo=react&logoColor=black" alt="React">
  <img src="https://img.shields.io/badge/-Express.js-000000?logo=express&logoColor=white" alt="Express.js">
  <img src="https://img.shields.io/badge/-Vite-646CFF?logo=vite&logoColor=white" alt="Vite">
  <img src="https://img.shields.io/badge/-Sharp-6c757d?logo=sharp&logoColor=white" alt="Sharp">
  <img src="https://img.shields.io/badge/-SVGO-4B32C3?logo=svgo&logoColor=white" alt="SVGO">
  <img src="https://img.shields.io/badge/-PSD-4B32C3?logo=adobe-photoshop&logoColor=white" alt="PSD">
  <img src="https://img.shields.io/badge/-Multer-000000?logo=multer&logoColor=white" alt="Multer">
  <img src="https://img.shields.io/badge/-Axios-000000?logo=axios&logoColor=white" alt="Axios">
  <img src="https://img.shields.io/badge/-ESLint-4B32C3?logo=eslint&logoColor=white" alt="ESLint">
  <img src="https://img.shields.io/badge/-npm-CB3837?logo=npm&logoColor=white" alt="npm">
</p>

-----

## Getting Started

To get a local copy up and running, follow these simple steps.

### Prerequisites

Please make sure you have Node.js and npm (Node Package Manager) installed.
Additionally, for full functionality, ImageMagick and Inkscape are required for certain image conversions (e.g., AI and PSD conversions).

  * **Node.js & npm**: Install from [Node.js official website](https://nodejs.org/).
  * **ImageMagick**: Download and install from [ImageMagick website](https://imagemagick.org/script/download.php).
  * **Inkscape**: Download and install from [Inkscape website](https://inkscape.org/release/).

### Installation

1.  Clone the repository:
    ```bash
    git clone [repository_url_here]
    ```
2.  Navigate to the project root directory:
    ```bash
    cd Converter-5eab401413612d6a4de27c5824218d0ac65be006
    ```
3.  Install NPM packages for both backend and frontend:
    ```bash
    cd backend
    npm install
    cd ../frontend
    npm install
    cd ..
    ```

## Usage

### Development

To run the project in development mode:

1.  Start the backend server:

    ```bash
    cd backend
    npm run dev
    ```

    (This will start the backend on `http://localhost:3000`)

2.  In a new terminal, start the frontend development server:

    ```bash
    cd frontend
    npm run dev
    ```

    (This will start the frontend on `http://localhost:5173` or similar)

### Build

To build the project for production:

1.  Build the backend:
    ```bash
    cd backend
    npm run build
    ```
2.  Build the frontend:
    ```bash
    cd frontend
    npm run build
    ```
    The built files will be located in the `backend/dist` and `frontend/dist` directories respectively.

### Deployment

This project can be deployed to static hosting services. For example, to deploy the frontend to GitHub Pages:

```bash
# In the frontend directory
cd frontend
npm run deploy
```

The homepage will be set according to your `package.json` configuration for `gh-pages`.

### Available Scripts

In the project directories, you can run:

**Backend Scripts:**

  * `npm start`: Starts the production server.
  * `npm run dev`: Starts the development server with `ts-node-dev`.
  * `npm run build`: Compiles TypeScript files to JavaScript.
  * `npm test`: Runs tests (currently not configured, outputs an error message).

**Frontend Scripts:**

  * `npm run dev`: Starts the Vite development server.
  * `npm run build`: Builds the app for production to the `dist` folder.
  * `npm run lint`: Lints the codebase using ESLint.
  * `npm run preview`: Serves the production build locally.
  * `npm run predeploy`: Runs the build script before deployment (part of `gh-pages` setup).
  * `npm run deploy`: Deploys the `dist` folder to GitHub Pages (part of `gh-pages` setup).

## File Structure

The main structure of the project is as follows:

```
.
├── .gitignore
├── README.md
├── backend/
│   ├── dist/                 # Compiled JavaScript files
│   │   ├── controllers/
│   │   ├── middleware/
│   │   ├── routers/
│   │   └── services/
│   ├── src/                  # Backend TypeScript source code
│   │   ├── controllers/      # Handlers for API routes
│   │   ├── middleware/       # Express middleware (e.g., upload handling)
│   │   ├── routers/          # API route definitions
│   │   └── services/         # Core logic for image conversion and AI processing
│   ├── package.json          # Backend dependencies and scripts
│   ├── package-lock.json
│   └── tsconfig.json         # TypeScript configuration for backend
└── frontend/
    ├── dist/                 # Built frontend assets
    ├── public/               # Static assets (e.g., vite.svg)
    ├── src/                  # React/TypeScript source code
    │   ├── assets/           # Application assets
    │   ├── components/       # Reusable React components
    │   │   ├── ConversionOptions.tsx
    │   │   ├── ConversionResult.tsx
    │   │   └── ImageUploader.tsx
    │   ├── App.css           # Main application CSS
    │   ├── App.tsx           # Main React application component
    │   └── main.tsx          # Entry point for React app
    ├── index.html            # Main HTML file
    ├── package.json          # Frontend dependencies and scripts
    ├── package-lock.json
    ├── eslint.config.js      # ESLint configuration for frontend
    ├── tsconfig.app.json     # TypeScript configuration for application code
    ├── tsconfig.json         # Main TypeScript configuration
    ├── tsconfig.node.json    # TypeScript configuration for Node environment
    └── vite.config.ts        # Vite build configuration
```

## License

This project is licensed under the MIT License and ISC License, reflecting the separate licensing of the frontend and backend components respectively.

[backend/package.json](https://www.google.com/search?q=uploaded:ramadharanuh/converter/Converter-5eab401413612d6a4de27c5824218d0ac65be006/backend/package.json)
[frontend/package.json](https://www.google.com/search?q=uploaded:ramadharanuh/converter/Converter-5eab401413612d6a4de27c5824218d0ac65be006/frontend/package.json)
