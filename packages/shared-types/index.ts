export interface TeamMember {
  id: string;
  championPool: string[];
  role: 'top' | 'jungle' | 'mid' | 'adc' | 'support';
}
