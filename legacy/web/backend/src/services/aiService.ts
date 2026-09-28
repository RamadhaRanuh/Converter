import { exec } from 'child_process';
import path from 'path';
import { promisify } from 'util';

class AIService {
    private execPromise = promisify(exec);
    async convertAI(inputPath: string, outputFormat: string) {
        const outputPath = this.generateOutputPath(inputPath, outputFormat);
        //console.log(outputPath, 'tes');
        // Using ImageMagick
        try {
            await this.execPromise(`convert ${inputPath} ${outputPath}`);    
            return outputPath;
        } catch (error: any) {
            console.log(`ImageMagick failed: ${error.message}`);
            try {
                await this.execPromise(`inkscape ${inputPath} --export-filename=${outputPath}`);
                return outputPath;
            } catch (inkscapeError) {
                throw new Error('Failed to convert AI file');
            }
        } 
    }

    generateOutputPath(inputPath: string, outputFormat: string) {
        const dirName = path.dirname(inputPath);
        const baseName = path.basename(inputPath, path.extname(inputPath));
        return path.join(dirName, `${baseName}.${outputFormat}`);
    }
}

export default new AIService();