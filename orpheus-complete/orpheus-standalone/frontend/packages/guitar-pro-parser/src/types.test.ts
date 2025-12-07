import { describe, it, expect, beforeEach } from 'vitest';
import { BinaryReader, BinaryWriter } from './types';

describe('types.ts - Binary Reader/Writer', () => {
  describe('BinaryReader', () => {
    describe('readByte()', () => {
      it('should read a single byte', () => {
        const buffer = new Uint8Array([0x42]).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.readByte()).toBe(0x42);
      });

      it('should read multiple bytes sequentially', () => {
        const buffer = new Uint8Array([0x10, 0x20, 0x30]).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.readByte()).toBe(0x10);
        expect(reader.readByte()).toBe(0x20);
        expect(reader.readByte()).toBe(0x30);
      });

      it('should increment offset after reading', () => {
        const buffer = new Uint8Array([0x10, 0x20]).buffer;
        const reader = new BinaryReader(buffer);

        reader.readByte();
        expect(reader.getOffset()).toBe(1);

        reader.readByte();
        expect(reader.getOffset()).toBe(2);
      });

      it('should read value 0', () => {
        const buffer = new Uint8Array([0x00]).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.readByte()).toBe(0);
      });

      it('should read max byte value (255)', () => {
        const buffer = new Uint8Array([0xFF]).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.readByte()).toBe(255);
      });
    });

    describe('readInt()', () => {
      it('should read 32-bit integer (little-endian)', () => {
        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        view.setInt32(0, 12345, true); // Little-endian

        const reader = new BinaryReader(buffer);
        expect(reader.readInt()).toBe(12345);
      });

      it('should read negative integer', () => {
        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        view.setInt32(0, -12345, true);

        const reader = new BinaryReader(buffer);
        expect(reader.readInt()).toBe(-12345);
      });

      it('should read zero', () => {
        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        view.setInt32(0, 0, true);

        const reader = new BinaryReader(buffer);
        expect(reader.readInt()).toBe(0);
      });

      it('should read max positive integer', () => {
        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        view.setInt32(0, 2147483647, true);

        const reader = new BinaryReader(buffer);
        expect(reader.readInt()).toBe(2147483647);
      });

      it('should read max negative integer', () => {
        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        view.setInt32(0, -2147483648, true);

        const reader = new BinaryReader(buffer);
        expect(reader.readInt()).toBe(-2147483648);
      });

      it('should increment offset by 4 bytes', () => {
        const buffer = new ArrayBuffer(4);
        const reader = new BinaryReader(buffer);

        reader.readInt();
        expect(reader.getOffset()).toBe(4);
      });

      it('should read multiple integers sequentially', () => {
        const buffer = new ArrayBuffer(12);
        const view = new DataView(buffer);
        view.setInt32(0, 100, true);
        view.setInt32(4, 200, true);
        view.setInt32(8, 300, true);

        const reader = new BinaryReader(buffer);

        expect(reader.readInt()).toBe(100);
        expect(reader.readInt()).toBe(200);
        expect(reader.readInt()).toBe(300);
      });
    });

    describe('readShort()', () => {
      it('should read 16-bit integer (little-endian)', () => {
        const buffer = new ArrayBuffer(2);
        const view = new DataView(buffer);
        view.setInt16(0, 1234, true);

        const reader = new BinaryReader(buffer);
        expect(reader.readShort()).toBe(1234);
      });

      it('should read negative short', () => {
        const buffer = new ArrayBuffer(2);
        const view = new DataView(buffer);
        view.setInt16(0, -1234, true);

        const reader = new BinaryReader(buffer);
        expect(reader.readShort()).toBe(-1234);
      });

      it('should read zero', () => {
        const buffer = new ArrayBuffer(2);
        const view = new DataView(buffer);
        view.setInt16(0, 0, true);

        const reader = new BinaryReader(buffer);
        expect(reader.readShort()).toBe(0);
      });

      it('should read max short value', () => {
        const buffer = new ArrayBuffer(2);
        const view = new DataView(buffer);
        view.setInt16(0, 32767, true);

        const reader = new BinaryReader(buffer);
        expect(reader.readShort()).toBe(32767);
      });

      it('should read min short value', () => {
        const buffer = new ArrayBuffer(2);
        const view = new DataView(buffer);
        view.setInt16(0, -32768, true);

        const reader = new BinaryReader(buffer);
        expect(reader.readShort()).toBe(-32768);
      });

      it('should increment offset by 2 bytes', () => {
        const buffer = new ArrayBuffer(2);
        const reader = new BinaryReader(buffer);

        reader.readShort();
        expect(reader.getOffset()).toBe(2);
      });

      it('should read multiple shorts sequentially', () => {
        const buffer = new ArrayBuffer(6);
        const view = new DataView(buffer);
        view.setInt16(0, 10, true);
        view.setInt16(2, 20, true);
        view.setInt16(4, 30, true);

        const reader = new BinaryReader(buffer);

        expect(reader.readShort()).toBe(10);
        expect(reader.readShort()).toBe(20);
        expect(reader.readShort()).toBe(30);
      });
    });

    describe('readString()', () => {
      it('should read ASCII string', () => {
        const buffer = new Uint8Array([72, 101, 108, 108, 111]).buffer; // "Hello"
        const reader = new BinaryReader(buffer);

        expect(reader.readString(5)).toBe('Hello');
      });

      it('should read empty string (length 0)', () => {
        const buffer = new Uint8Array([]).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.readString(0)).toBe('');
      });

      it('should read single character', () => {
        const buffer = new Uint8Array([65]).buffer; // "A"
        const reader = new BinaryReader(buffer);

        expect(reader.readString(1)).toBe('A');
      });

      it('should read numbers as characters', () => {
        const buffer = new Uint8Array([49, 50, 51]).buffer; // "123"
        const reader = new BinaryReader(buffer);

        expect(reader.readString(3)).toBe('123');
      });

      it('should read special characters', () => {
        const buffer = new Uint8Array([33, 64, 35, 36, 37]).buffer; // "!@#$%"
        const reader = new BinaryReader(buffer);

        expect(reader.readString(5)).toBe('!@#$%');
      });

      it('should increment offset by string length', () => {
        const buffer = new Uint8Array([72, 101, 108, 108, 111]).buffer;
        const reader = new BinaryReader(buffer);

        reader.readString(5);
        expect(reader.getOffset()).toBe(5);
      });

      it('should read partial string from longer buffer', () => {
        const buffer = new Uint8Array([72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100]).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.readString(5)).toBe('Hello');
      });
    });

    describe('readIntSizeString()', () => {
      it('should read length-prefixed string (int size)', () => {
        const buffer = new ArrayBuffer(9);
        const view = new DataView(buffer);
        view.setInt32(0, 5, true); // Length = 5
        view.setUint8(4, 72); // 'H'
        view.setUint8(5, 101); // 'e'
        view.setUint8(6, 108); // 'l'
        view.setUint8(7, 108); // 'l'
        view.setUint8(8, 111); // 'o'

        const reader = new BinaryReader(buffer);
        expect(reader.readIntSizeString()).toBe('Hello');
      });

      it('should read empty string with length 0', () => {
        const buffer = new ArrayBuffer(4);
        const view = new DataView(buffer);
        view.setInt32(0, 0, true);

        const reader = new BinaryReader(buffer);
        expect(reader.readIntSizeString()).toBe('');
      });

      it('should handle longer strings', () => {
        const text = 'Guitar Pro File Format';
        const buffer = new ArrayBuffer(4 + text.length);
        const view = new DataView(buffer);
        view.setInt32(0, text.length, true);

        for (let i = 0; i < text.length; i++) {
          view.setUint8(4 + i, text.charCodeAt(i));
        }

        const reader = new BinaryReader(buffer);
        expect(reader.readIntSizeString()).toBe(text);
      });

      it('should increment offset correctly', () => {
        const buffer = new ArrayBuffer(9);
        const view = new DataView(buffer);
        view.setInt32(0, 5, true);

        const reader = new BinaryReader(buffer);
        reader.readIntSizeString();

        expect(reader.getOffset()).toBe(9); // 4 bytes (length) + 5 bytes (string)
      });
    });

    describe('readByteSizeString()', () => {
      it('should read length-prefixed string (byte size)', () => {
        const buffer = new Uint8Array([5, 72, 101, 108, 108, 111]).buffer; // Length=5, "Hello"
        const reader = new BinaryReader(buffer);

        expect(reader.readByteSizeString()).toBe('Hello');
      });

      it('should read empty string with length 0', () => {
        const buffer = new Uint8Array([0]).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.readByteSizeString()).toBe('');
      });

      it('should handle max byte size (255)', () => {
        const buffer = new Uint8Array(256);
        buffer[0] = 255; // Length
        for (let i = 1; i <= 255; i++) {
          buffer[i] = 65; // 'A'
        }

        const reader = new BinaryReader(buffer.buffer);
        const result = reader.readByteSizeString();

        expect(result.length).toBe(255);
        expect(result).toBe('A'.repeat(255));
      });

      it('should increment offset correctly', () => {
        const buffer = new Uint8Array([5, 72, 101, 108, 108, 111]).buffer;
        const reader = new BinaryReader(buffer);

        reader.readByteSizeString();
        expect(reader.getOffset()).toBe(6); // 1 byte (length) + 5 bytes (string)
      });
    });

    describe('skip()', () => {
      it('should skip specified number of bytes', () => {
        const buffer = new Uint8Array(10).buffer;
        const reader = new BinaryReader(buffer);

        reader.skip(5);
        expect(reader.getOffset()).toBe(5);
      });

      it('should skip 0 bytes', () => {
        const buffer = new Uint8Array(10).buffer;
        const reader = new BinaryReader(buffer);

        reader.skip(0);
        expect(reader.getOffset()).toBe(0);
      });

      it('should skip multiple times', () => {
        const buffer = new Uint8Array(20).buffer;
        const reader = new BinaryReader(buffer);

        reader.skip(5);
        reader.skip(3);
        reader.skip(2);

        expect(reader.getOffset()).toBe(10);
      });

      it('should allow reading after skip', () => {
        const buffer = new Uint8Array([0, 0, 0, 0, 42]).buffer;
        const reader = new BinaryReader(buffer);

        reader.skip(4);
        expect(reader.readByte()).toBe(42);
      });
    });

    describe('getOffset()', () => {
      it('should return initial offset of 0', () => {
        const buffer = new Uint8Array(10).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.getOffset()).toBe(0);
      });

      it('should return current offset after reads', () => {
        const buffer = new ArrayBuffer(10);
        const reader = new BinaryReader(buffer);

        reader.readByte(); // offset = 1
        reader.readShort(); // offset = 3
        reader.readInt(); // offset = 7

        expect(reader.getOffset()).toBe(7);
      });
    });

    describe('setOffset()', () => {
      it('should set offset to specified position', () => {
        const buffer = new Uint8Array(10).buffer;
        const reader = new BinaryReader(buffer);

        reader.setOffset(5);
        expect(reader.getOffset()).toBe(5);
      });

      it('should allow seeking back to start', () => {
        const buffer = new Uint8Array(10).buffer;
        const reader = new BinaryReader(buffer);

        reader.skip(5);
        reader.setOffset(0);

        expect(reader.getOffset()).toBe(0);
      });

      it('should allow re-reading from specific position', () => {
        const buffer = new Uint8Array([10, 20, 30, 40, 50]).buffer;
        const reader = new BinaryReader(buffer);

        reader.readByte(); // Read 10
        reader.readByte(); // Read 20
        reader.setOffset(0); // Go back to start

        expect(reader.readByte()).toBe(10); // Re-read first byte
      });

      it('should allow jumping to end', () => {
        const buffer = new Uint8Array(10).buffer;
        const reader = new BinaryReader(buffer);

        reader.setOffset(10);
        expect(reader.getOffset()).toBe(10);
      });
    });

    describe('hasMore()', () => {
      it('should return true when buffer has unread data', () => {
        const buffer = new Uint8Array(10).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.hasMore()).toBe(true);
      });

      it('should return false when at end of buffer', () => {
        const buffer = new Uint8Array(5).buffer;
        const reader = new BinaryReader(buffer);

        reader.skip(5);
        expect(reader.hasMore()).toBe(false);
      });

      it('should return false for empty buffer', () => {
        const buffer = new Uint8Array(0).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.hasMore()).toBe(false);
      });

      it('should update as bytes are read', () => {
        const buffer = new Uint8Array(3).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.hasMore()).toBe(true);
        reader.readByte();
        expect(reader.hasMore()).toBe(true);
        reader.readByte();
        expect(reader.hasMore()).toBe(true);
        reader.readByte();
        expect(reader.hasMore()).toBe(false);
      });
    });

    describe('mixed operations', () => {
      it('should handle complex read sequence', () => {
        const buffer = new ArrayBuffer(20);
        const view = new DataView(buffer);
        view.setUint8(0, 0x10); // Byte
        view.setInt16(1, 1000, true); // Short
        view.setInt32(3, 50000, true); // Int
        view.setUint8(7, 72); // 'H'
        view.setUint8(8, 105); // 'i'

        const reader = new BinaryReader(buffer);

        expect(reader.readByte()).toBe(0x10);
        expect(reader.readShort()).toBe(1000);
        expect(reader.readInt()).toBe(50000);
        expect(reader.readString(2)).toBe('Hi');
        expect(reader.getOffset()).toBe(9);
      });

      it('should handle skip and read pattern', () => {
        const buffer = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).buffer;
        const reader = new BinaryReader(buffer);

        expect(reader.readByte()).toBe(1);
        reader.skip(3);
        expect(reader.readByte()).toBe(5);
        reader.skip(2);
        expect(reader.readByte()).toBe(8);
      });

      it('should handle seek and re-read pattern', () => {
        const buffer = new Uint8Array([100, 200]).buffer;
        const reader = new BinaryReader(buffer);

        const first = reader.readByte();
        reader.setOffset(0);
        const second = reader.readByte();

        expect(first).toBe(second);
      });
    });
  });

  describe('BinaryWriter', () => {
    let writer: BinaryWriter;

    beforeEach(() => {
      writer = new BinaryWriter();
    });

    describe('writeByte()', () => {
      it('should write a single byte', () => {
        writer.writeByte(0x42);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getUint8(0)).toBe(0x42);
      });

      it('should write multiple bytes', () => {
        writer.writeByte(0x10);
        writer.writeByte(0x20);
        writer.writeByte(0x30);

        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getUint8(0)).toBe(0x10);
        expect(view.getUint8(1)).toBe(0x20);
        expect(view.getUint8(2)).toBe(0x30);
      });

      it('should write byte value 0', () => {
        writer.writeByte(0);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getUint8(0)).toBe(0);
      });

      it('should write byte value 255', () => {
        writer.writeByte(255);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getUint8(0)).toBe(255);
      });

      it('should truncate values > 255 to byte range', () => {
        writer.writeByte(256); // Should be 0 (256 & 0xFF)
        writer.writeByte(257); // Should be 1 (257 & 0xFF)

        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getUint8(0)).toBe(0);
        expect(view.getUint8(1)).toBe(1);
      });
    });

    describe('writeInt()', () => {
      it('should write 32-bit integer (little-endian)', () => {
        writer.writeInt(12345);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getInt32(0, true)).toBe(12345);
      });

      it('should write negative integer', () => {
        writer.writeInt(-12345);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getInt32(0, true)).toBe(-12345);
      });

      it('should write zero', () => {
        writer.writeInt(0);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getInt32(0, true)).toBe(0);
      });

      it('should write max positive integer', () => {
        writer.writeInt(2147483647);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getInt32(0, true)).toBe(2147483647);
      });

      it('should write multiple integers', () => {
        writer.writeInt(100);
        writer.writeInt(200);
        writer.writeInt(300);

        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getInt32(0, true)).toBe(100);
        expect(view.getInt32(4, true)).toBe(200);
        expect(view.getInt32(8, true)).toBe(300);
      });

      it('should write 4 bytes', () => {
        writer.writeInt(12345);
        const buffer = writer.getBuffer();

        expect(buffer.byteLength).toBe(4);
      });
    });

    describe('writeShort()', () => {
      it('should write 16-bit integer (little-endian)', () => {
        writer.writeShort(1234);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getInt16(0, true)).toBe(1234);
      });

      it('should write negative short', () => {
        writer.writeShort(-1234);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getInt16(0, true)).toBe(-1234);
      });

      it('should write zero', () => {
        writer.writeShort(0);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getInt16(0, true)).toBe(0);
      });

      it('should write multiple shorts', () => {
        writer.writeShort(10);
        writer.writeShort(20);
        writer.writeShort(30);

        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getInt16(0, true)).toBe(10);
        expect(view.getInt16(2, true)).toBe(20);
        expect(view.getInt16(4, true)).toBe(30);
      });

      it('should write 2 bytes', () => {
        writer.writeShort(1234);
        const buffer = writer.getBuffer();

        expect(buffer.byteLength).toBe(2);
      });
    });

    describe('writeString()', () => {
      it('should write fixed-length string', () => {
        writer.writeString('Hello', 5);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(String.fromCharCode(view.getUint8(0))).toBe('H');
        expect(String.fromCharCode(view.getUint8(1))).toBe('e');
        expect(String.fromCharCode(view.getUint8(2))).toBe('l');
        expect(String.fromCharCode(view.getUint8(3))).toBe('l');
        expect(String.fromCharCode(view.getUint8(4))).toBe('o');
      });

      it('should pad with zeros if string is shorter than length', () => {
        writer.writeString('Hi', 5);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(String.fromCharCode(view.getUint8(0))).toBe('H');
        expect(String.fromCharCode(view.getUint8(1))).toBe('i');
        expect(view.getUint8(2)).toBe(0);
        expect(view.getUint8(3)).toBe(0);
        expect(view.getUint8(4)).toBe(0);
      });

      it('should truncate if string is longer than length', () => {
        writer.writeString('HelloWorld', 5);
        const buffer = writer.getBuffer();
        const bytes: number[] = [];

        for (let i = 0; i < buffer.byteLength; i++) {
          bytes.push(new DataView(buffer).getUint8(i));
        }

        const result = String.fromCharCode(...bytes);
        expect(result).toBe('Hello');
      });

      it('should write empty string with zeros', () => {
        writer.writeString('', 3);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getUint8(0)).toBe(0);
        expect(view.getUint8(1)).toBe(0);
        expect(view.getUint8(2)).toBe(0);
      });

      it('should write numbers as characters', () => {
        writer.writeString('123', 3);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getUint8(0)).toBe('1'.charCodeAt(0));
        expect(view.getUint8(1)).toBe('2'.charCodeAt(0));
        expect(view.getUint8(2)).toBe('3'.charCodeAt(0));
      });
    });

    describe('writeIntSizeString()', () => {
      it('should write length-prefixed string (int size)', () => {
        writer.writeIntSizeString('Hello');
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        // Check length prefix (4 bytes)
        expect(view.getInt32(0, true)).toBe(5);

        // Check string content
        expect(String.fromCharCode(view.getUint8(4))).toBe('H');
        expect(String.fromCharCode(view.getUint8(5))).toBe('e');
        expect(String.fromCharCode(view.getUint8(6))).toBe('l');
        expect(String.fromCharCode(view.getUint8(7))).toBe('l');
        expect(String.fromCharCode(view.getUint8(8))).toBe('o');
      });

      it('should write empty string with length 0', () => {
        writer.writeIntSizeString('');
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getInt32(0, true)).toBe(0);
        expect(buffer.byteLength).toBe(4);
      });

      it('should write long string correctly', () => {
        const text = 'Guitar Pro File Format';
        writer.writeIntSizeString(text);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getInt32(0, true)).toBe(text.length);
        expect(buffer.byteLength).toBe(4 + text.length);
      });
    });

    describe('writeByteSizeString()', () => {
      it('should write length-prefixed string (byte size)', () => {
        writer.writeByteSizeString('Hello');
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        // Check length prefix (1 byte)
        expect(view.getUint8(0)).toBe(5);

        // Check string content
        expect(String.fromCharCode(view.getUint8(1))).toBe('H');
        expect(String.fromCharCode(view.getUint8(2))).toBe('e');
        expect(String.fromCharCode(view.getUint8(3))).toBe('l');
        expect(String.fromCharCode(view.getUint8(4))).toBe('l');
        expect(String.fromCharCode(view.getUint8(5))).toBe('o');
      });

      it('should write empty string with length 0', () => {
        writer.writeByteSizeString('');
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getUint8(0)).toBe(0);
        expect(buffer.byteLength).toBe(1);
      });

      it('should write max byte size string (255 chars)', () => {
        const text = 'A'.repeat(255);
        writer.writeByteSizeString(text);
        const buffer = writer.getBuffer();
        const view = new DataView(buffer);

        expect(view.getUint8(0)).toBe(255);
        expect(buffer.byteLength).toBe(256); // 1 byte length + 255 bytes content
      });
    });

    describe('getBuffer()', () => {
      it('should return ArrayBuffer', () => {
        writer.writeByte(42);
        const buffer = writer.getBuffer();

        expect(buffer).toBeInstanceOf(ArrayBuffer);
      });

      it('should return correct buffer size', () => {
        writer.writeByte(1); // 1 byte
        writer.writeShort(2); // 2 bytes
        writer.writeInt(3); // 4 bytes

        const buffer = writer.getBuffer();
        expect(buffer.byteLength).toBe(7);
      });

      it('should return empty buffer for new writer', () => {
        const buffer = writer.getBuffer();
        expect(buffer.byteLength).toBe(0);
      });
    });

    describe('roundtrip with BinaryReader', () => {
      it('should write and read byte', () => {
        writer.writeByte(0x42);
        const buffer = writer.getBuffer();

        const reader = new BinaryReader(buffer);
        expect(reader.readByte()).toBe(0x42);
      });

      it('should write and read integer', () => {
        writer.writeInt(12345);
        const buffer = writer.getBuffer();

        const reader = new BinaryReader(buffer);
        expect(reader.readInt()).toBe(12345);
      });

      it('should write and read short', () => {
        writer.writeShort(1234);
        const buffer = writer.getBuffer();

        const reader = new BinaryReader(buffer);
        expect(reader.readShort()).toBe(1234);
      });

      it('should write and read string', () => {
        writer.writeString('Hello', 5);
        const buffer = writer.getBuffer();

        const reader = new BinaryReader(buffer);
        expect(reader.readString(5)).toBe('Hello');
      });

      it('should write and read int-size string', () => {
        writer.writeIntSizeString('Guitar Pro');
        const buffer = writer.getBuffer();

        const reader = new BinaryReader(buffer);
        expect(reader.readIntSizeString()).toBe('Guitar Pro');
      });

      it('should write and read byte-size string', () => {
        writer.writeByteSizeString('Test');
        const buffer = writer.getBuffer();

        const reader = new BinaryReader(buffer);
        expect(reader.readByteSizeString()).toBe('Test');
      });

      it('should handle complex write/read sequence', () => {
        writer.writeByte(0x10);
        writer.writeShort(1000);
        writer.writeInt(50000);
        writer.writeString('GP', 2);
        writer.writeIntSizeString('Guitar Pro 7');

        const buffer = writer.getBuffer();
        const reader = new BinaryReader(buffer);

        expect(reader.readByte()).toBe(0x10);
        expect(reader.readShort()).toBe(1000);
        expect(reader.readInt()).toBe(50000);
        expect(reader.readString(2)).toBe('GP');
        expect(reader.readIntSizeString()).toBe('Guitar Pro 7');
      });

      it('should handle multiple integers roundtrip', () => {
        const values = [100, 200, 300, 400, 500];

        values.forEach(v => writer.writeInt(v));
        const buffer = writer.getBuffer();

        const reader = new BinaryReader(buffer);
        values.forEach(v => {
          expect(reader.readInt()).toBe(v);
        });
      });

      it('should handle negative values roundtrip', () => {
        writer.writeInt(-12345);
        writer.writeShort(-1234);

        const buffer = writer.getBuffer();
        const reader = new BinaryReader(buffer);

        expect(reader.readInt()).toBe(-12345);
        expect(reader.readShort()).toBe(-1234);
      });
    });
  });
});
