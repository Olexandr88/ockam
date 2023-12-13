import SwiftUI

struct SentInvitations: View {
    @State private var isHovered = false
    @State private var isOpen = false
    @ObservedObject var state: ApplicationState

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: HorizontalSpacingUnit) {
                Text("Sent invitations")
                    .font(.body)
                    .padding(.horizontal, HorizontalSpacingUnit)
                Spacer()
                Image(systemName: "chevron.right")
                    .rotationEffect(
                        isOpen ? Angle.degrees(90.0) : Angle.degrees(0), anchor: .center
                    )
                    .padding([.trailing], HorizontalSpacingUnit)
            }
            .frame(height: VerticalSpacingUnit*3)
            .contentShape(Rectangle())
            .onTapGesture {
                withAnimation {
                    isOpen = !isOpen
                }
            }
            .onHover { hover in
                isHovered = hover
            }
            .background(isHovered ? Color.gray.opacity(0.25) : Color.clear)
            .cornerRadius(4)
            .padding(.horizontal, WindowBorderSize)

            if isOpen {
                Divider()
                HStack(spacing: 0) {
                    VStack(alignment: .leading, spacing: 0) {
                        ForEach(state.sent_invitations) { invitation in
                            Text(invitation.email)
                                .frame(height: VerticalSpacingUnit*2)
                        }
                    }
                    .padding(.horizontal, HorizontalSpacingUnit*2)
                    Spacer()
                }
                .padding(.vertical, VerticalSpacingUnit)
                .background(HierarchicalShapeStyle.quinary)
                Divider()
            }
        }
    }
}


struct SentInvitations_Previews: PreviewProvider {
    @State static var state = swift_demo_application_state()

    static var previews: some View {
        SentInvitations(state: state)
            .frame(width: 320, height: 200)
    }
}
