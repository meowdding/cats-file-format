package me.owdding.cats.impl;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;

public final class DataReader {

    private final byte[] data;
    private int position;

    public DataReader(byte[] data) {
        this.data = data;
        this.position = 0;
    }

    private void ensureAvailable(int length) {
        if (position + length > data.length) {
            throw new IndexOutOfBoundsException("Not enough data available to read");
        }
    }

    public byte readByte() {
        ensureAvailable(1);

        return data[position++];
    }

    public short readUnsignedByte() {
        ensureAvailable(1);

        return (short) (data[position++] & 0xFF);
    }

    public int readUnsignedShort() {
        ensureAvailable(2);

        int value = ((data[position] & 0xFF) << 8) | (data[position + 1] & 0xFF);
        position += 2;
        return value;
    }

    public int readInt() {
        ensureAvailable(4);

        int value = ((data[position] & 0xFF) << 24) |
                ((data[position + 1] & 0xFF) << 16) |
                ((data[position + 2] & 0xFF) << 8) |
                (data[position + 3] & 0xFF);
        position += 4;
        return value;
    }

    public String readString() {
        var length = readUnsignedByte();

        ensureAvailable(length);

        var strBytes = Arrays.copyOfRange(data, position, position + length);
        position += length;

        return new String(strBytes, StandardCharsets.US_ASCII);
    }
}
