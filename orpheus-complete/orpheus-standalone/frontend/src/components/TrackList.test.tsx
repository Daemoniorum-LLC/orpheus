import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { TrackList } from './TrackList';

describe('TrackList', () => {
  const mockTracks = [
    {
      id: 'track-1',
      name: 'Track 1',
      trackNumber: 1,
      volume: 0.8,
      pan: 0,
      muted: false,
      soloed: false,
      recordArmed: false,
      peakLevel: 0.5,
      rmsLevel: 0.3,
    },
    {
      id: 'track-2',
      name: 'Track 2',
      trackNumber: 2,
      volume: 0.7,
      pan: 0.5,
      muted: false,
      soloed: false,
      recordArmed: false,
      peakLevel: 0.4,
      rmsLevel: 0.2,
    },
  ];

  const mockOnAddTrack = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('should render track list container', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);
    expect(screen.getByTestId('track-list')).toBeInTheDocument();
  });

  it('should render add track button', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);
    expect(screen.getByTestId('add-track-button')).toBeInTheDocument();
  });

  it('should render all tracks', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    expect(screen.getByText('Track 1')).toBeInTheDocument();
    expect(screen.getByText('Track 2')).toBeInTheDocument();
  });

  it('should render tracks in order by track number', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    const trackItems = screen.getAllByTestId('track-item');
    expect(trackItems).toHaveLength(2);
  });

  it('should open add track dialog when button clicked', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    const addButton = screen.getByTestId('add-track-button');
    fireEvent.click(addButton);

    expect(screen.getByTestId('add-track-dialog')).toBeInTheDocument();
  });

  it('should render track name input in dialog', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    fireEvent.click(screen.getByTestId('add-track-button'));

    expect(screen.getByTestId('track-name-input')).toBeInTheDocument();
  });

  it('should render instrument select in dialog', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    fireEvent.click(screen.getByTestId('add-track-button'));

    expect(screen.getByTestId('track-instrument-select')).toBeInTheDocument();
  });

  it('should render create track submit button', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    fireEvent.click(screen.getByTestId('add-track-button'));

    expect(screen.getByTestId('create-track-submit')).toBeInTheDocument();
  });

  it('should update track name input', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    fireEvent.click(screen.getByTestId('add-track-button'));

    const nameInput = screen.getByTestId('track-name-input') as HTMLInputElement;
    fireEvent.change(nameInput, { target: { value: 'New Track' } });

    expect(nameInput.value).toBe('New Track');
  });

  it('should update instrument select', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    fireEvent.click(screen.getByTestId('add-track-button'));

    const instrumentSelect = screen.getByTestId('track-instrument-select') as HTMLSelectElement;
    fireEvent.change(instrumentSelect, { target: { value: 'drums' } });

    expect(instrumentSelect.value).toBe('drums');
  });

  it('should call onAddTrack when form submitted', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    fireEvent.click(screen.getByTestId('add-track-button'));

    fireEvent.change(screen.getByTestId('track-name-input'), {
      target: { value: 'Bass Track' },
    });
    fireEvent.change(screen.getByTestId('track-instrument-select'), {
      target: { value: 'bass' },
    });

    fireEvent.click(screen.getByTestId('create-track-submit'));

    expect(mockOnAddTrack).toHaveBeenCalledWith({
      name: 'Bass Track',
      instrument: 'bass',
    });
  });

  it('should close dialog after track created', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    fireEvent.click(screen.getByTestId('add-track-button'));
    fireEvent.change(screen.getByTestId('track-name-input'), {
      target: { value: 'Test' },
    });
    fireEvent.click(screen.getByTestId('create-track-submit'));

    expect(screen.queryByTestId('add-track-dialog')).not.toBeInTheDocument();
  });

  it('should close dialog when cancel clicked', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    fireEvent.click(screen.getByTestId('add-track-button'));

    const dialog = screen.getByTestId('add-track-dialog');
    fireEvent.click(dialog);

    expect(screen.queryByTestId('add-track-dialog')).not.toBeInTheDocument();
  });

  it('should include all instrument options', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    fireEvent.click(screen.getByTestId('add-track-button'));

    const select = screen.getByTestId('track-instrument-select');
    const options = select.querySelectorAll('option');
    const values = Array.from(options).map(opt => opt.getAttribute('value'));

    expect(values).toContain('electric-guitar');
    expect(values).toContain('acoustic-guitar');
    expect(values).toContain('bass');
    expect(values).toContain('drums');
    expect(values).toContain('piano');
    expect(values).toContain('synth');
    expect(values).toContain('vocals');
  });

  it('should render with empty track list', () => {
    render(<TrackList tracks={[]} onAddTrack={mockOnAddTrack} />);

    expect(screen.getByTestId('track-list')).toBeInTheDocument();
    const trackItems = screen.queryAllByTestId('track-item');
    expect(trackItems).toHaveLength(0);
  });

  it('should handle very long track names', () => {
    const longNameTrack = {
      ...mockTracks[0],
      name: 'This is a very long track name that might overflow',
    };

    render(<TrackList tracks={[longNameTrack]} onAddTrack={mockOnAddTrack} />);

    expect(screen.getByText('This is a very long track name that might overflow')).toBeInTheDocument();
  });

  it('should not submit form with empty track name', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    fireEvent.click(screen.getByTestId('add-track-button'));
    fireEvent.click(screen.getByTestId('create-track-submit'));

    expect(mockOnAddTrack).not.toHaveBeenCalled();
  });

  it('should reset form after submission', () => {
    render(<TrackList tracks={mockTracks} onAddTrack={mockOnAddTrack} />);

    fireEvent.click(screen.getByTestId('add-track-button'));
    fireEvent.change(screen.getByTestId('track-name-input'), {
      target: { value: 'Test Track' },
    });
    fireEvent.click(screen.getByTestId('create-track-submit'));

    // Reopen dialog
    fireEvent.click(screen.getByTestId('add-track-button'));

    const nameInput = screen.getByTestId('track-name-input') as HTMLInputElement;
    expect(nameInput.value).toBe('');
  });
});
