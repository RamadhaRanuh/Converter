<h1 align="center">Convertly – A sleek, modern play on "convert"</h1>

<p align="center">A fast, offline desktop converter for images, designs and documents. Drop files in, pick a format, done.</p>

### Built with the tools and technologies:

<p align="center">
  <img src="https://img.shields.io/badge/-Rust-000000?logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/-egui-4B32C3?logo=rust&logoColor=white" alt="egui">
  <img src="https://img.shields.io/badge/-Typst-239DAD?logo=typst&logoColor=white" alt="Typst">
  <img src="https://img.shields.io/badge/-Windows-0078D4?logo=windows&logoColor=white" alt="Windows">
  <img src="https://img.shields.io/badge/-GitHub%20Actions-2088FF?logo=githubactions&logoColor=white" alt="GitHub Actions">
</p>

## Download

Get the latest Windows version (10/11, 64-bit) from the **[Releases page](https://github.com/RamadhaRanuh/Converter/releases/latest)**:

- **`Converter_…_x64-setup.exe`**: installs for your user only (no admin prompt) and adds a Start menu entry.
- **`Converter-…-x64-portable.zip`**: unzip anywhere and run `Converter.exe`. Includes the `converter-cli.exe` command-line tool.

The app isn't code-signed yet, so on first launch Windows SmartScreen may say "Windows protected your PC".
Click **More info → Run anyway**. This only happens once.

## What it converts

| From | To |
|---|---|
| JPG, PNG, WebP, BMP, TIFF, GIF | any of those, PSD, SVG (traced), PDF |
| HEIC (iPhone photos) | JPG, PNG, WebP, BMP, TIFF, GIF, PDF |
| PSD, AI (Illustrator) | JPG, PNG, WebP, BMP, TIFF, GIF, PDF |
| SVG | images, optimized SVG, PDF |
| PDF | one image per page, split into pages |
| Word, PowerPoint, Excel | PDF |
| Markdown | PDF, HTML |

It can also **combine** several images and PDFs into one PDF. Everything runs on your machine; nothing is uploaded.

## Build from source

The app lives in [`converter/`](converter/README.md), a Rust workspace with the conversion core, the desktop
window, a command-line tool and a benchmark. See its README for build steps, the optional HEIC setup and speed
numbers.

## Legacy web app (archived)

The original React/Express web app is kept for reference in [`legacy/web/`](legacy/web/). The desktop app
replaces it; it is no longer maintained and needs ImageMagick and Inkscape installed.

### Getting Started

To get a local copy up and running, follow these simple steps.

#### Prerequisites

Please make sure you have Node.js and npm (Node Package Manager) installed.
Additionally, for full functionality, ImageMagick and Inkscape are required for certain image conversions (e.g., AI and PSD conversions).

  * **Node.js & npm**: Install from [Node.js official website](https://nodejs.org/).
  * **ImageMagick**: Download and install from [ImageMagick website](https://imagemagick.org/script/download.php).
  * **Inkscape**: Download and install from [Inkscape website](https://inkscape.org/release/).

#### Installation

1.  Clone the repository:
    ```bash
    git clone https://github.com/RamadhaRanuh/Converter.git
    ```
2.  Navigate to the project root directory:
    ```bash
    cd Converter
    ```
3.  Install NPM packages for both backend and frontend:
    ```bash
    cd legacy/web/backend
    npm install
    cd ../frontend
    npm install
    cd ../../..
    ```

### Usage

#### Development

To run the project in development mode:

1.  Start the backend server:

    ```bash
    cd legacy/web/backend
    npm run dev
    ```

    (This will start the backend on `http://localhost:3000`)

2.  In a new terminal, start the frontend development server:

    ```bash
    cd legacy/web/frontend
    npm run dev
    ```

    (This will start the frontend on `http://localhost:5173` or similar)

#### Build

To build the project for production:

1.  Build the backend:
    ```bash
    cd legacy/web/backend
    npm run build
    ```
2.  Build the frontend:
    ```bash
    cd legacy/web/frontend
    npm run build
    ```
    The built files will be located in the `legacy/web/backend/dist` and `legacy/web/frontend/dist` directories respectively.

#### Deployment

This project can be deployed to static hosting services. For example, to deploy the frontend to GitHub Pages:

```bash
# In the frontend directory
cd legacy/web/frontend
npm run deploy
```

The homepage will be set according to your `package.json` configuration for `gh-pages`.

#### Available Scripts

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

### File Structure

The web app's structure, under `legacy/web/`:

```
legacy/web/
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

The desktop app in `converter/` is licensed under MIT OR Apache-2.0 (see `converter/LICENSE-MIT` and
`converter/LICENSE-APACHE`). The archived web app keeps its original licences: MIT for the frontend and ISC for
the backend.
