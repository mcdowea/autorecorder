#!/bin/bash
# WAV to MP3 Converter Script
# Requires ffmpeg to be installed

if ! command -v ffmpeg &> /dev/null; then
    echo "Error: ffmpeg not found. Please install ffmpeg first."
    echo ""
    echo "Installation:"
    echo "  Ubuntu/Debian: sudo apt-get install ffmpeg"
    echo "  macOS:         brew install ffmpeg"
    echo "  Windows:       Download from https://ffmpeg.org/download.html"
    exit 1
fi

# Default values
INPUT_DIR="recordings"
OUTPUT_DIR="recordings/mp3"
BITRATE="128k"
QUALITY="2"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -i|--input)
            INPUT_DIR="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -b|--bitrate)
            BITRATE="$2"
            shift 2
            ;;
        -q|--quality)
            QUALITY="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -i, --input DIR     Input directory (default: recordings)"
            echo "  -o, --output DIR    Output directory (default: recordings/mp3)"
            echo "  -b, --bitrate RATE  MP3 bitrate (default: 128k)"
            echo "  -q, --quality NUM   MP3 quality 0-9 (default: 2)"
            echo "  -h, --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                                    # Convert all WAV files"
            echo "  $0 -b 192k -q 1                      # High quality conversion"
            echo "  $0 -i ./my-recordings -o ./output    # Custom directories"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use -h or --help for usage information"
            exit 1
            ;;
    esac
done

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Convert all WAV files
count=0
for wav_file in "$INPUT_DIR"/*.wav; do
    if [ -f "$wav_file" ]; then
        filename=$(basename "$wav_file" .wav)
        mp3_file="$OUTPUT_DIR/${filename}.mp3"
        
        echo "Converting: $wav_file -> $mp3_file"
        ffmpeg -i "$wav_file" -vn -ar 44100 -ac 1 -b:a "$BITRATE" -q:a "$QUALITY" "$mp3_file" -y 2>&1 | grep -v "^ffmpeg version" | grep -v "^  lib" | grep -v "^  configuration" | grep -v "^  built"
        
        if [ $? -eq 0 ]; then
            echo "✓ Converted successfully"
            ((count++))
        else
            echo "✗ Conversion failed"
        fi
        echo ""
    fi
done

if [ $count -eq 0 ]; then
    echo "No WAV files found in $INPUT_DIR"
else
    echo "======================================"
    echo "Conversion complete!"
    echo "Converted $count file(s)"
    echo "Output directory: $OUTPUT_DIR"
    echo "======================================"
fi
