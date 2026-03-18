// Stub for profile management — ready to wire up useQuery when auth lands.
// When signed in, replace `profiles` with: useQuery(['profiles'], fetchProfiles)

export interface Profile {
  id: string
  name: string
}

export function useProfile() {
  // TODO: replace with useQuery(['profiles'], fetchProfiles) once auth is wired
  const profiles: Profile[] = []
  const activeProfile: Profile | null = null

  return { profiles, activeProfile }
}
